use std::{env, fmt::Display, io::Cursor, sync::Arc};

use anyhow::{Context, Error, Result};
use osu_db::Replay;
use reqwest::Client;
use rosu_pp::{Beatmap, BeatmapExt};
use rosu_v2::prelude::{Beatmap as Map, Beatmapset, GameMode, GameMods, Osu};
use serenity::{
    http::Http,
    model::{
        channel::Message,
        id::{ChannelId, UserId},
    },
    prelude::{RwLock, TypeMap},
};
use tokio::{
    fs::{self, DirEntry, File},
    io::AsyncWriteExt,
    process::Command,
};
use zip::ZipArchive;

use crate::{
    replay_queue::ReplayStatus,
    util::{levenshtein_similarity, CustomUploadApi},
    ReplayHandler, ReplayQueue, ServerSettings,
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
    pub input_channel: ChannelId,
    pub output_channel: ChannelId,
    pub path: String,
    pub replay: Replay,
    pub time_points: Option<TimePoints>,
    pub user: UserId,
}

#[derive(Copy, Clone, Debug)]
pub struct TimePoints {
    pub start: Option<u32>,
    pub end: Option<u32>,
}

impl TimePoints {
    pub fn parse_single(s: &str) -> Result<u32, &'static str> {
        let mut iter = s.split(':').map(str::parse);

        match (iter.next(), iter.next()) {
            (Some(Ok(minutes)), Some(Ok(seconds @ 0..=59))) => Ok(minutes * 60 + seconds),
            (Some(Ok(_)), Some(Ok(_))) => Err("Seconds must be between 0 and 60!"),
            (Some(Ok(seconds)), None) => Ok(seconds),
            _ => Err("A value you supplied is not a number!"),
        }
    }
}

pub async fn process_replay(osu: Osu, http: Arc<Http>, client: Client, queue: Arc<ReplayQueue>) {
    let url = env::var("CUSTOM_UPLOAD_URL")
        .context("missing env variable `CUSTOM_UPLOAD_URL`")
        .unwrap();

    let secret_key = env::var("CUSTOM_UPLOAD_SECRET")
        .context("missing env variable `CUSTOM_UPLOAD_SECRET`")
        .unwrap();

    let uploader = CustomUploadApi::new(url, secret_key)
        .await
        .context("failed to create custom upload api wrapper")
        .unwrap();

    loop {
        let Data {
            input_channel,
            output_channel: replay_channel,
            path: replay_path,
            replay: replay_file,
            time_points,
            user: replay_user,
        } = queue.peek().await;

        let mapset = match replay_file.beatmap_hash.as_deref() {
            Some(hash) => match osu.beatmap().checksum(hash).await {
                Ok(Map { mapset, .. }) => match mapset {
                    Some(mapset) => mapset,
                    None => {
                        warn!("missing mapset in map");

                        send_error_message(
                            &http,
                            input_channel,
                            replay_user,
                            "the mapset is missing in the map",
                        )
                        .await;

                        queue.reset_peek().await;
                        continue;
                    }
                },
                Err(why) => {
                    let err = Error::new(why)
                        .context(format!("failed to request map with hash `{hash}`"));
                    warn!("{err:?}");

                    send_error_message(
                        &http,
                        input_channel,
                        replay_user,
                        format!("failed to get the map with hash: `{hash}`"),
                    )
                    .await;

                    queue.reset_peek().await;
                    continue;
                }
            },
            None => {
                warn!("No hash in replay requested by user {replay_user}");

                send_error_message(
                    &http,
                    input_channel,
                    replay_user,
                    "couldn't find hash in your replay file",
                )
                .await;

                queue.reset_peek().await;
                continue;
            }
        };

        let mapset_id = mapset.mapset_id;
        info!("Started map download");
        queue.set_status(ReplayStatus::Downloading).await;

        if let Err(why) = download_mapset(mapset_id, &client).await {
            warn!("{:?}", why);

            send_error_message(
                &http,
                input_channel,
                replay_user,
                format!("failed to download map: {why}"),
            )
            .await;

            queue.reset_peek().await;
            continue;
        }

        info!("Finished map download");

        let settings = if path_exists(format!("../danser/settings/{replay_user}.json")).await {
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
                warn!("replay path `{replay_path}` has an unexpected form");

                send_error_message(
                    &http,
                    input_channel,
                    replay_user,
                    "there was an error resolving the beatmap path",
                )
                .await;

                queue.reset_peek().await;
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

        if let Some(time_points) = time_points {
            if let Some(start) = time_points.start {
                command.args(["-start", &start.to_string()]);
            }

            if let Some(end) = time_points.end {
                command.args(["-end", &end.to_string()]);
            }
        }

        info!("Started replay parsing");
        queue.set_status(ReplayStatus::Processing).await;

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
                    input_channel,
                    replay_user,
                    format!("failed to parse replay: {err}"),
                )
                .await;

                queue.reset_peek().await;
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
                    input_channel,
                    replay_user,
                    "there was an error reading the log file",
                )
                .await;

                queue.reset_peek().await;
                continue;
            }
        };

        let map_path = format!("../Songs/{}/{}", mapset_id, map_osu_file);
        let filepath = format!("../Replays/{}.mp4", filename);

        let video_title = match create_title(&replay_file, map_path, &mapset).await {
            Ok(title) => title,
            Err(why) => {
                warn!("{:?}", why.context("failed to create title"));

                send_error_message(
                    &http,
                    input_channel,
                    replay_user,
                    "there was an error while trying to create the video title",
                )
                .await;

                queue.reset_peek().await;
                continue;
            }
        };

        info!("Started upload to shisha.mezo.xyz");
        queue.set_status(ReplayStatus::Uploading).await;

        let link = match uploader
            .upload_video(video_title, replay_user, &filepath)
            .await
        {
            Ok(response) => {
                if response.error == 1 {
                    warn!("failed to upload: {}", response.text);
                    send_error_message(
                        &http,
                        input_channel,
                        replay_user,
                        format!("failed to upload: `{}`", response.text).as_str(),
                    )
                    .await;

                    queue.reset_peek().await;
                    continue;
                } else {
                    response.text
                }
            }
            Err(why) => {
                warn!("{:?}", why.context("failed to upload file"));

                send_error_message(
                    &http,
                    input_channel,
                    replay_user,
                    "failed to upload to custom uploader",
                )
                .await;

                queue.reset_peek().await;
                continue;
            }
        };

        info!("Finished upload to shisha.mezo.xyz");

        let content = format!("<@{replay_user}> your replay is ready! {link}");

        let msg_fut = replay_channel.send_message(&http, |m| m.content(content));

        if let Err(why) = msg_fut.await {
            let err = Error::new(why).context("failed to send video link");
            warn!("{:?}", err);
        }

        queue.reset_peek().await;
    }
}

pub async fn parse_attachment_replay(
    msg: &Message,
    ctx_data: &RwLock<TypeMap>,
    time_points: Option<TimePoints>,
) -> AttachmentParseResult {
    let attachment = match msg.attachments.last() {
        Some(a) if matches!(a.filename.split('.').last(), Some("osr")) => a,
        Some(_) | None => return Ok(AttachmentParseSuccess::NothingToDo),
    };

    let guild_id;
    let channel_opt;
    let output_channel;

    if msg.is_private() {
        output_channel = msg.channel_id;
    } else {
        guild_id = match msg.guild_id {
            Some(guild_id) => guild_id,
            None => return Ok(AttachmentParseSuccess::NothingToDo),
        };

        channel_opt = {
            let data = ctx_data.read().await;
            let settings = data.get::<ServerSettings>().unwrap();

            settings
                .servers
                .get(&guild_id)
                .filter(|s| s.input_channel == msg.channel_id)
                .map(|s| s.output_channel)
        };

        output_channel = match channel_opt {
            Some(channel_id) => channel_id,
            None => return Ok(AttachmentParseSuccess::NothingToDo),
        };
    }

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
                "failed to create file: {err:?}"
            )));
        }
    };

    match file.write_all(&bytes).await {
        Ok(()) => (),
        Err(err) => {
            warn!("failed writing to file");
            return Err(AttachmentParseError::Other(anyhow!(
                "failed writing to file: {err:?}"
            )));
        }
    };

    let replay_data = Data {
        input_channel: msg.channel_id,
        output_channel,
        path: format!("../Downloads/{}", attachment.filename),
        replay,
        time_points,
        user: msg.author.id,
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
    let out_path = format!("../Songs/{mapset_id}");
    let url = format!("https://kitsu.moe/d/{mapset_id}");

    let kitsu = match download_mapset_(url, &out_path, client).await {
        Ok(_) => return Ok(()),
        Err(why) => why,
    };
    debug!("Using secondary mirror");
    let url = format!("https://chimu.moe/d/{mapset_id}");

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
        Err(err) => return Err(anyhow!("failed to GET using: {url}, error: {err}")),
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
                "failed to extract zip archive at `{out_path}`, error: {err}"
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
        "[{stars}⭐] {player} | {map_title} {mods}{acc}%",
        mods = if &mods_str == "NM" {
            String::new()
        } else {
            format!("+{mods_str} ")
        },
    );

    Ok(title)
}

async fn get_beatmap_osu_file(mapset_id: u32) -> Result<String> {
    let file = match fs::read_to_string("../danser/danser.log").await {
        Ok(file) => file,
        Err(err) => return Err(anyhow!("failed to read danser logs: {err}")),
    };

    let line = if let Some(l) = file.lines().find(|line| line.contains("Playing:")) {
        l
    } else {
        return Err(anyhow!("expected `Playing:` in danser logs"));
    };

    let map_without_artist = if let Some(m) = line.splitn(4, ' ').last() {
        m
    } else {
        return Err(anyhow!(
            "expected at least 5 words in danser log line `{line}`"
        ));
    };

    let items_dir = format!("../Songs/{}", mapset_id);

    let mut items = match fs::read_dir(&items_dir).await {
        Ok(items) => items,
        Err(err) => {
            return Err(anyhow!(
                "failed to read items dir at `{items_dir}`, error: {err}"
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

        debug!("COMPARING: {map_without_artist} WITH: {item_file_name}");

        let similarity = levenshtein_similarity(map_without_artist, &item_file_name);

        if similarity > max_similarity {
            max_similarity = similarity;
            final_file_name = item_file_name.into_owned();
        }
    }

    debug!("FINAL TITLE: {final_file_name} SIMILARITY: {max_similarity}");

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
                * (replay.count_katsu as u32 + replay.count_geki as u32))
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

    let line = if let Some(l) = file.lines().find(|line| line.contains("Playing:")) {
        l
    } else {
        return Err(anyhow!("expected `Playing:` in danser logs"));
    };

    let map_without_artist = if let Some(m) = line.splitn(4, ' ').last() {
        m
    } else {
        return Err(anyhow!(
            "expected at least 5 words in danser log line `{line}`"
        ));
    };

    Ok(map_without_artist.to_string())
}

async fn send_error_message(
    http: &Http,
    channel: ChannelId,
    replay_user: UserId,
    content: impl Display,
) {
    if let Err(err) = channel
        .send_message(&http, |m| m.content(format!("<@{replay_user}>, {content}")))
        .await
    {
        warn!("Couldn't send error message to discord: {err}");
    }
}
