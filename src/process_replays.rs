use std::{env, io::Cursor, sync::Arc, time::Duration};

use anyhow::{Context, Error, Result};
use osu_db::Replay;
use reqwest::Client;
use rosu_pp::{Beatmap, BeatmapExt};
use rosu_v2::prelude::{Beatmap as Map, Beatmapset, GameMode, GameMods, Osu};
use serenity::{
    client::bridge::gateway::ShardMessenger,
    http::Http,
    model::{
        channel::Message,
        id::{ChannelId, UserId},
        prelude::Activity,
    },
    prelude::{RwLock, TypeMap},
};
use tokio::{
    fs::{self, DirEntry, File},
    io::AsyncWriteExt,
    process::Command,
    time::{self, interval, MissedTickBehavior},
};
use zip::ZipArchive;

use crate::{
    streamable_wrapper::StreamableApi, util::levenshtein_similarity, ReplayHandler, ReplayQueue,
    ServerSettings, DEFAULT_PREFIX,
};

pub enum AttachmentParseSuccess {
    NothingToDo,
    BeingProcessed,
}

#[derive(Debug, thiserror::Error)]
pub enum AttachmentParseError {
    #[error("failed to download attachment")]
    Download(#[from] serenity::prelude::SerenityError),
    #[error(transparent)]
    Other(#[from] Error),
    #[error("failed to parse replay")]
    Parsing(#[from] osu_db::Error),
}

type AttachmentParseResult = Result<AttachmentParseSuccess, AttachmentParseError>;

#[derive(Clone, Debug)]
pub struct Data {
    pub path: String,
    pub replay: Replay,
    pub channel: ChannelId,
    pub user: UserId,
    pub replay_params: String,
    shard: ShardMessenger,
    server_prefixes: Vec<String>,
}

pub async fn process_replay(osu: Osu, http: Arc<Http>, client: Client, queue: Arc<ReplayQueue>) {
    let username = env::var("STREAMABLE_USERNAME")
        .context("missing env variable `STREAMABLE_USERNAME`")
        .unwrap();

    let password = env::var("STREAMABLE_PASSWORD")
        .context("missing env variable `STREAMABLE_PASSWORD`")
        .unwrap();

    let streamable = StreamableApi::new(username, password)
        .await
        .context("failed to create streamable api wrapper")
        .unwrap();

    loop {
        let replay_data = queue.front().await;

        let replay_path = replay_data.path;
        let replay_file = replay_data.replay;
        let replay_user = replay_data.user;
        let replay_channel = replay_data.channel;
        let shard = replay_data.shard;
        let server_prefixes = replay_data.server_prefixes;

        let mapset = match replay_file.beatmap_hash.as_deref() {
            Some(hash) => match osu.beatmap().checksum(hash).await {
                Ok(Map { mapset, .. }) => match mapset {
                    Some(mapset) => mapset,
                    None => {
                        warn!("missing mapset in map");

                        send_error_message(
                            &http,
                            replay_channel,
                            replay_user,
                            "the mapset is missing in the map",
                        )
                        .await;

                        shard.set_activity(Some(Activity::watching("!!help - Waiting for replay")));

                        queue.default_status().await;
                        queue.pop().await;
                        continue;
                    }
                },
                Err(why) => {
                    let err = Error::new(why)
                        .context(format!("failed to request map with hash `{}`", hash));
                    warn!("{:?}", err);

                    send_error_message(
                        &http,
                        replay_channel,
                        replay_user,
                        format!("failed to get the map with hash: `{}`", &hash).as_str(),
                    )
                    .await;

                    shard.set_activity(Some(Activity::watching("!!help - Waiting for replay")));

                    queue.default_status().await;
                    queue.pop().await;
                    continue;
                }
            },
            None => {
                warn!("No hash in replay requested by user {}", replay_user);

                send_error_message(
                    &http,
                    replay_channel,
                    replay_user,
                    "couldn't find hash in your replay file",
                )
                .await;

                shard.set_activity(Some(Activity::watching("!!help - Waiting for replay")));

                queue.default_status().await;
                queue.pop().await;
                continue;
            }
        };

        shard.set_activity(Some(Activity::watching("!!help - Downloading replay")));

        let mapset_id = mapset.mapset_id;
        info!("Started map download");
        queue.update_status().await;

        if let Err(why) = download_mapset(mapset_id, &client).await {
            warn!("{:?}", why);

            send_error_message(
                &http,
                replay_channel,
                replay_user,
                format!("failed to download map: {}", why).as_str(),
            )
            .await;

            shard.set_activity(Some(Activity::watching("!!help - Waiting for replay")));

            queue.default_status().await;
            queue.pop().await;
            continue;
        }

        info!("Finished map download");

        let settings = if path_exists(format!("../danser/settings/{}.json", replay_user)).await {
            replay_user.to_string()
        } else {
            "default".to_string()
        };

        let filename_opt = replay_path
            .split('/')
            .last()
            .and_then(|file| file.split('.').next());

        let filename = match filename_opt {
            Some(name) => name,
            None => {
                warn!("replay path `{}` has an unexpected form", replay_path);

                send_error_message(
                    &http,
                    replay_channel,
                    replay_user,
                    "there was an error resolving the beatmap path",
                )
                .await;

                shard.set_activity(Some(Activity::watching("!!help - Waiting for replay")));

                queue.default_status().await;
                queue.pop().await;
                continue;
            }
        };

        let mut command = Command::new("../danser/danser");

        command
            .arg(format!("-replay={}", replay_path))
            .arg("-record")
            .arg(format!("-settings={}", settings))
            .arg("-quickstart")
            .arg(format!("-out={}", filename));

        if check_server_prefix(server_prefixes, &replay_data.replay_params) {
            let params = replay_data.replay_params.split(' ').collect::<Vec<&str>>();
            command.args(["-start", params[1]]);
            if params.len() == 3 {
                command.args(["-end", params[2]]);
            }
        }

        shard.set_activity(Some(Activity::watching("!!help - Parsing replay")));
        info!("Started replay parsing");
        queue.update_status().await;

        match command.output().await {
            Ok(output) => {
                if let Ok(stdout) = std::str::from_utf8(&output.stdout) {
                    debug!("stdout: {}", stdout);
                }

                if let Ok(stderr) = std::str::from_utf8(&output.stderr) {
                    debug!("stderr: {}", stderr);
                }
            }
            Err(why) => {
                let err = Error::new(why).context("failed to get command output");
                warn!("{:?}", err);

                send_error_message(
                    &http,
                    replay_channel,
                    replay_user,
                    format!("failed to parse replay: {}", err).as_str(),
                )
                .await;

                shard.set_activity(Some(Activity::watching("!!help - Waiting for replay")));

                queue.default_status().await;
                queue.pop().await;
                continue;
            }
        }

        info!("Finished replay parsing");

        let map_osu_file = match get_beatmap_osu_file(mapset_id).await {
            Ok(osu_file) => osu_file,
            Err(why) => {
                warn!("{:?}", why.context("failed to get map_osu_file"));

                send_error_message(
                    &http,
                    replay_channel,
                    replay_user,
                    "there was an error reading the log file",
                )
                .await;

                shard.set_activity(Some(Activity::watching("!!help - Waiting for replay")));

                queue.default_status().await;
                queue.pop().await;
                continue;
            }
        };

        let map_path = format!("../Songs/{}/{}", mapset_id, map_osu_file);
        let filepath = format!("../Replays/{}.mp4", filename);

        let streamable_title = match create_title(&replay_file, map_path, &mapset).await {
            Ok(title) => title,
            Err(why) => {
                warn!("{:?}", why.context("failed to create title"));

                send_error_message(
                    &http,
                    replay_channel,
                    replay_user,
                    "there was an error while trying to create the streamable title",
                )
                .await;

                shard.set_activity(Some(Activity::watching("!!help - Waiting for replay")));

                queue.default_status().await;
                queue.pop().await;
                continue;
            }
        };

        let activity = "!!help - Uploading replay to streamable";
        shard.set_activity(Some(Activity::watching(activity)));

        info!("Started upload to streamable");
        queue.update_status().await;

        let shortcode = match streamable.upload_video(streamable_title, &filepath).await {
            Ok(response) => response.shortcode,
            Err(why) => {
                warn!("{:?}", why.context("failed to upload file"));

                send_error_message(
                    &http,
                    replay_channel,
                    replay_user,
                    "failed to upload to streamable",
                )
                .await;

                shard.set_activity(Some(Activity::watching("!!help - Waiting for replay")));

                queue.default_status().await;
                queue.pop().await;
                continue;
            }
        };

        tokio::select! {
            res = await_video_ready(&streamable, &shortcode) => {
                if res.is_err() {
                    warn!(
                        "Got too many errors while trying to retrieve video's ready status, \
                        abort and go to next..."
                    );

                    send_error_message(
                        &http,
                        replay_channel,
                        replay_user,
                        "there was an error while trying to retrieve the video's ready status",
                    )
                    .await;

                    shard.set_activity(Some(Activity::watching("!!help - Waiting for replay")));

                    queue.default_status().await;
                    queue.pop().await;
                    continue;
                }
            }
            _ = time::sleep(Duration::from_secs(300)) => {
                warn!("Failed to upload video within 5 minutes, abort and go to next...");

                send_error_message(
                    &http,
                    replay_channel,
                    replay_user,
                    "failed to upload the replay within 5 minutes",
                )
                .await;

                shard.set_activity(Some(Activity::watching("!!help - Waiting for replay")));

                queue.default_status().await;
                queue.pop().await;
                continue;
            }
        }

        info!("Finished upload to streamable");

        let content =
            format!("<@{replay_user}> your replay is ready! https://streamable.com/{shortcode}");

        let msg_fut = replay_channel.send_message(&http, |m| m.content(content));

        shard.set_activity(Some(Activity::watching("!!help - Waiting for replay")));
        if let Err(why) = msg_fut.await {
            let err = Error::new(why).context("failed to send streamable link");
            warn!("{:?}", err);
        }

        queue.update_status().await;
        queue.pop().await;
    }
}

async fn await_video_ready(streamable: &StreamableApi, shortcode: &str) -> Result<(), ()> {
    let mut interval = interval(Duration::from_secs(5));
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

    let mut attempt: u8 = 0;
    const ATTEMPTS: u8 = 10;

    loop {
        interval.tick().await;

        let status = match streamable.check_status_code(shortcode).await {
            Ok(status) => {
                attempt = 0;

                status
            }
            Err(why) => {
                warn!("failed to get status code on attempt #{attempt}/{ATTEMPTS}: {why}");
                attempt += 1;

                if attempt == ATTEMPTS {
                    return Err(());
                }

                continue;
            }
        };

        if status == 2 {
            return Ok(());
        }
    }
}

pub async fn parse_attachment_replay(
    msg: &Message,
    shard_messenger: ShardMessenger,
    ctx_data: &RwLock<TypeMap>,
) -> AttachmentParseResult {
    let attachment = match msg.attachments.last() {
        Some(a) if matches!(a.filename.split('.').last(), Some("osr")) => a,
        Some(_) | None => return Ok(AttachmentParseSuccess::NothingToDo),
    };

    let guild_id = match msg.guild_id {
        Some(guild_id) => guild_id,
        None => return Ok(AttachmentParseSuccess::NothingToDo),
    };

    let channel_opt = {
        let data = ctx_data.read().await;
        let settings = data.get::<ServerSettings>().unwrap();

        settings
            .servers
            .get(&guild_id)
            .filter(|s| s.replay_channel == msg.channel_id)
            .map(|s| s.output_channel)
    };

    let prefixes = {
        let data = ctx_data.read().await;
        let settings = data.get::<ServerSettings>().unwrap();

        settings.servers.get(&guild_id).map(|s| s.prefixes.clone())
    };

    let output_channel = match channel_opt {
        Some(channel_id) => channel_id,
        None => return Ok(AttachmentParseSuccess::NothingToDo),
    };

    let bytes = match attachment.download().await {
        Ok(bytes) => bytes,
        Err(err) => {
            warn!("download error: {err}");
            return Err(AttachmentParseError::Download(err));
        }
    };

    let replay = match osu_db::Replay::from_bytes(&bytes) {
        Ok(replay) => replay,
        Err(err) => {
            warn!("osu_db replay error: {err}");
            return Err(AttachmentParseError::Parsing(err));
        }
    };

    let mut file = match File::create(format!("../Downloads/{}", &attachment.filename)).await {
        Ok(file) => file,
        Err(err) => {
            warn!("failed to create file: {err}");
            return Err(AttachmentParseError::Other(anyhow!(
                "failed to create file: {:?}",
                err
            )));
        }
    };

    match file.write_all(&bytes).await {
        Ok(()) => (),
        Err(err) => {
            warn!("failed writing to file");
            return Err(AttachmentParseError::Other(anyhow!(
                "failed writing to file: {:?}",
                err
            )));
        }
    };

    let replay_data = Data {
        path: format!("../Downloads/{}", &attachment.filename),
        replay,
        channel: output_channel,
        user: msg.author.id,
        replay_params: msg.content.to_string(),
        shard: shard_messenger,
        server_prefixes: prefixes.unwrap_or_else(|| vec![DEFAULT_PREFIX.to_string()]),
    };

    ctx_data
        .read()
        .await
        .get::<ReplayHandler>()
        .unwrap()
        .push(replay_data)
        .await;

    Ok(AttachmentParseSuccess::BeingProcessed)
}

async fn path_exists(path: String) -> bool {
    fs::metadata(path).await.is_ok()
}

#[derive(Debug, thiserror::Error)]
#[error(
    "failed to download mapset\n\
    <https://chimu.moe> error: {}\n\
    <https://kitsu.moe> error: {}",
    kitsu,
    chimu
)]
struct MapsetDownloadError {
    kitsu: Error,
    chimu: Error,
}

async fn download_mapset(mapset_id: u32, client: &Client) -> Result<()> {
    let out_path = format!("../Songs/{}", mapset_id);
    let url = format!("https://kitsu.moe/d/{}", mapset_id);

    let kitsu = match download_mapset_(url, &out_path, client).await {
        Ok(_) => return Ok(()),
        Err(why) => why,
    };
    debug!("Using secondary mirror");
    let url = format!("https://chimu.moe/d/{}", mapset_id);

    let chimu = match download_mapset_(url, &out_path, client).await {
        Ok(_) => return Ok(()),
        Err(why) => why,
    };

    Err(MapsetDownloadError { kitsu, chimu }.into())
}

async fn download_mapset_(url: String, out_path: &str, client: &Client) -> Result<()> {
    let bytes = match client.get(&url).send().await {
        Ok(resp) => match resp.bytes().await {
            Ok(bytes) => bytes,
            Err(err) => return Err(anyhow!("failed to read bytes: {err}")),
        },
        Err(err) => {
            return Err(anyhow!(
                "failed to GET using: {}, error: {}",
                url.as_str(),
                err
            ))
        }
    };

    let cursor = Cursor::new(bytes);

    let mut archive = match ZipArchive::new(cursor) {
        Ok(archive) => archive,
        Err(err) => return Err(anyhow!("failed to create zip archive: {err}")),
    };

    match archive.extract(out_path) {
        Ok(()) => (),
        Err(err) => {
            return Err(anyhow!(
                "failed to extract zip archive at `{}`, error: {}",
                out_path,
                err
            ))
        }
    };

    Ok(())
}

async fn create_title(replay: &Replay, map_path: String, _mapset: &Beatmapset) -> Result<String> {
    let mods = replay.mods.bits();

    let stars = match Beatmap::from_path(&map_path).await {
        Ok(beatmap) => beatmap.stars(mods, None).stars(),
        Err(err) => return Err(anyhow!("failed to get stars: {err}")),
    };

    let mods_str = GameMods::from_bits(mods).unwrap_or_default().to_string();
    let stars = (stars * 100.0).round() / 100.0;
    let player = replay.player_name.as_deref().unwrap_or_default();
    let map_title = get_title().await.unwrap();
    let acc = accuracy(replay, GameMode::STD);

    let title = format!(
        "[{}⭐] {} | {}{}{}%",
        stars,
        player,
        map_title,
        if &mods_str == "NM" {
            " ".to_owned()
        } else {
            format!(" +{} ", mods_str)
        },
        acc
    );

    Ok(title)
}

async fn get_beatmap_osu_file(mapset_id: u32) -> Result<String> {
    let file = match fs::read_to_string("../danser/danser.log").await {
        Ok(file) => file,
        Err(err) => return Err(anyhow!("failed to read danser logs: {err}")),
    };

    let line;

    if let Some(l) = file.lines().find(|line| line.contains("Playing:")) {
        line = l;
    } else {
        return Err(anyhow!("expected `Playing:` in danser logs"));
    }

    let map_without_artist;

    if let Some(m) = line.splitn(4, ' ').last() {
        map_without_artist = m;
    } else {
        return Err(anyhow!(
            "expected at least 5 words in danser log line `{}`",
            line
        ));
    }

    let items_dir = format!("../Songs/{}", mapset_id);

    let mut items = match fs::read_dir(&items_dir).await {
        Ok(items) => items,
        Err(err) => {
            return Err(anyhow!(
                "failed to read items dir at `{}`, error: {}",
                items_dir,
                err
            ))
        }
    };

    let mut correct_items: Vec<DirEntry> = Vec::new();

    loop {
        match items.next_entry().await {
            Ok(Some(entry)) => {
                if entry.file_name().to_str().unwrap().ends_with(".osu") {
                    correct_items.push(entry);
                }
            }
            Ok(None) => break,
            Err(err) => {
                return Err(anyhow!(
                    "there was an error while trying to read the files: {err}"
                ))
            }
        }
    }

    let mut max_similarity: f32 = 0.0;
    let mut final_file_name = String::new();

    for item in correct_items {
        let file_name = item.file_name();
        let item_file_name = file_name.to_string_lossy();

        debug!("COMPARING: {} WITH: {}", map_without_artist, item_file_name);

        let similarity = levenshtein_similarity(map_without_artist, &item_file_name);

        if similarity > max_similarity {
            max_similarity = similarity;
            final_file_name = item_file_name.into_owned();
        }
    }

    debug!(
        "FINAL TITLE: {} SIMILARITY: {}",
        final_file_name, max_similarity
    );

    Ok(final_file_name)
}

fn accuracy(replay: &Replay, mode: GameMode) -> f32 {
    let amount_objects = total_hits(replay, mode) as f32;

    let (numerator, denumerator) = match mode {
        GameMode::TKO => (
            0.5 * replay.count_100 as f32 + replay.count_300 as f32,
            amount_objects,
        ),
        GameMode::CTB => (
            (replay.count_300 + replay.count_100 + replay.count_50) as f32,
            amount_objects,
        ),
        GameMode::STD | GameMode::MNA => {
            let mut n = (replay.count_50 as u32 * 50
                + replay.count_100 as u32 * 100
                + replay.count_300 as u32 * 300) as f32;

            n += ((mode == GameMode::MNA) as u32
                * (replay.count_katsu * 200 + replay.count_geki * 300) as u32)
                as f32;

            (n, amount_objects * 300.0)
        }
    };

    (10_000.0 * numerator / denumerator).round() / 100.0
}

fn total_hits(replay: &Replay, mode: GameMode) -> u32 {
    let mut amount = (replay.count_300 + replay.count_100 + replay.count_miss) as u32;

    if mode != GameMode::TKO {
        amount += replay.count_50 as u32;

        if mode != GameMode::STD {
            amount += replay.count_katsu as u32;
            amount += (mode != GameMode::CTB) as u32 * replay.count_geki as u32;
        }
    }

    amount
}

async fn get_title() -> Result<String> {
    let file = match fs::read_to_string("../danser/danser.log").await {
        Ok(file) => file,
        Err(err) => return Err(anyhow!("failed to read danser logs: {err}")),
    };

    let line;

    if let Some(l) = file.lines().find(|line| line.contains("Playing:")) {
        line = l;
    } else {
        return Err(anyhow!("expected `Playing:` in danser logs"));
    }

    let map_without_artist;

    if let Some(m) = line.splitn(4, ' ').last() {
        map_without_artist = m;
    } else {
        return Err(anyhow!(
            "expected at least 5 words in danser log line `{}`",
            line
        ));
    }

    Ok(map_without_artist.to_string())
}

fn check_server_prefix(server_prefixes: Vec<String>, params: &str) -> bool {
    server_prefixes
        .iter()
        .any(|p| params.starts_with(p) && params[p.len()..].starts_with("start"))
}

async fn send_error_message(
    http: &Arc<Http>,
    replay_channel: ChannelId,
    replay_user: UserId,
    content: &str,
) {
    if let Err(err) = replay_channel
        .send_message(&http, |m| {
            m.content(format!("<@{}>, {}", replay_user, content))
        })
        .await
    {
        warn!("Couldn't send error message to discord: {}", err);
    }
}
