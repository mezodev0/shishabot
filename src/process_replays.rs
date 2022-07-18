use std::{env, io::Cursor, mem, sync::Arc, time::Duration};

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
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    time::{self, interval, MissedTickBehavior},
};
use zip::ZipArchive;

use crate::{streamable_wrapper::StreamableApi, ServerSettings, DEFAULT_PREFIX};

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

pub struct Data {
    path: String,
    replay: Replay,
    channel: ChannelId,
    user: UserId,
    replay_params: String,
    shard: ShardMessenger,
    server_prefixes: Vec<String>,
}

pub async fn process_replay(
    mut receiver: UnboundedReceiver<Data>,
    osu: Osu,
    http: Arc<Http>,
    client: Client,
) {
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

    while let Some(replay_data) = receiver.recv().await {
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
                        continue;
                    }
                },
                Err(why) => {
                    let err = Error::new(why)
                        .context(format!("failed to request map with hash `{}`", hash));

                    warn!("{:?}", err);
                    continue;
                }
            },
            None => {
                warn!("No hash in replay requested by user {}", replay_user);
                continue;
            }
        };

        shard.set_activity(Some(Activity::watching("!!help - Downloading replay")));

        let mapset_id = mapset.mapset_id;
        info!("Started map download");

        if let Err(why) = download_mapset(mapset_id, &client).await {
            warn!("{:?}", why);
            if let Err(err) = replay_channel
                .send_message(&http, |m| {
                    m.content(format!(
                        "<@{}>, failed to download map: {}",
                        replay_user, why
                    ))
                })
                .await
            {
                warn!("Couldn't send error message to discord: {}", err);
            }
            shard.set_activity(Some(Activity::watching("!!help - Waiting for replay")));
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
                if let Err(why) = replay_channel
                    .send_message(&http, |m| {
                        m.content(format!(
                            "<@{}>, failed to parse replay: {}",
                            replay_user, err
                        ))
                    })
                    .await
                {
                    warn!("Failed to send error message to discord: {}", why);
                }
                shard.set_activity(Some(Activity::watching("!!help - Waiting for replay")));
                continue;
            }
        }

        info!("Finished replay parsing");

        let map_osu_file = match get_beatmap_osu_file(mapset_id).await {
            Ok(osu_file) => osu_file,
            Err(why) => {
                warn!("{:?}", why.context("failed to get map_osu_file"));
                if let Err(err) = replay_channel
                    .send_message(&http, |m| {
                        m.content(format!(
                            "<@{}>, the version the mirrors do not match the replay",
                            replay_user
                        ))
                    })
                    .await
                {
                    warn!("failed to send message: {}", err);
                }
                shard.set_activity(Some(Activity::watching("!!help - Waiting for replay")));
                continue;
            }
        };

        let map_path = format!("../Songs/{}/{}", mapset_id, map_osu_file);
        let filepath = format!("../Replays/{}.mp4", filename);

        let streamable_title = match create_title(&replay_file, map_path, &mapset).await {
            Ok(title) => title,
            Err(why) => {
                warn!("{:?}", why.context("failed to create title"));
                continue;
            }
        };

        let activity = "!!help - Uploading replay to streamable";
        shard.set_activity(Some(Activity::watching(activity)));

        info!("Started upload to streamable");

        let shortcode = match streamable.upload_video(streamable_title, &filepath).await {
            Ok(response) => response.shortcode,
            Err(why) => {
                warn!("{:?}", why.context("failed to upload file"));

                if let Err(err) = replay_channel
                    .send_message(&http, |m| {
                        m.content(format!("<@{replay_user}>, failed to upload to streamable"))
                    })
                    .await
                {
                    warn!("Failed to send error message to discord: {}", err);
                }

                shard.set_activity(Some(Activity::watching("!!help - Waiting for replay")));
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

                    continue;
                }
            }
            _ = time::sleep(Duration::from_secs(300)) => {
                warn!("Failed to upload video within 5 minutes, abort and go to next...");
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
    sender: &UnboundedSender<Data>,
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

    let bytes = attachment.download().await?;
    let replay = osu_db::Replay::from_bytes(&bytes)?;

    let mut file = File::create(format!("../Downloads/{}", &attachment.filename))
        .await
        .context("failed to create file")?;

    file.write_all(&bytes)
        .await
        .context("failed writing to file")?;

    let replay_data = Data {
        path: format!("../Downloads/{}", &attachment.filename),
        replay,
        channel: output_channel,
        user: msg.author.id,
        replay_params: msg.content.to_string(),
        shard: shard_messenger,
        server_prefixes: prefixes.unwrap_or_else(|| vec![DEFAULT_PREFIX.to_string()]),
    };

    if let Err(why) = sender.send(replay_data) {
        warn!("failed to send: {}", why);
    }

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
    let bytes = client.get(url).send().await?.bytes().await?;
    let cursor = Cursor::new(bytes);

    let mut archive = ZipArchive::new(cursor).context("failed to create zip archive")?;

    archive
        .extract(out_path)
        .with_context(|| format!("failed to extract zip archive at `{}`", out_path))?;

    Ok(())
}

async fn create_title(replay: &Replay, map_path: String, _mapset: &Beatmapset) -> Result<String> {
    let mods = replay.mods.bits();

    let stars = Beatmap::from_path(&map_path)
        .await
        .with_context(|| format!("failed to parse map `{}`", map_path))?
        .stars(mods, None)
        .stars();

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
    let file = fs::read_to_string("../danser/danser.log")
        .await
        .context("failed to read danser logs")?;

    let line = file
        .lines()
        .find(|line| line.contains("Playing:"))
        .ok_or_else(|| anyhow!("expected `Playing:` in danser logs"))?;

    let map_without_artist = line
        .splitn(4, ' ')
        .last()
        .ok_or_else(|| anyhow!("expected at least 5 words in danser log line `{}`", line))?;

    let items_dir = format!("../Songs/{}", mapset_id);

    let mut items = fs::read_dir(&items_dir)
        .await
        .with_context(|| format!("failed to read items dir at `{}`", items_dir))?;

    let mut correct_items: Vec<DirEntry> = Vec::new();

    while let Some(entry) = items.next_entry().await? {
        if entry.file_name().to_str().unwrap().ends_with(".osu") {
            correct_items.push(entry);
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

pub fn levenshtein_similarity(word_a: &str, word_b: &str) -> f32 {
    let (dist, len) = levenshtein_distance(word_a, word_b);

    (len - dist) as f32 / len as f32
}

macro_rules! get {
    ($slice:ident[$idx:expr]) => {
        unsafe { *$slice.get_unchecked($idx) }
    };
}

macro_rules! set {
    ($slice:ident[$idx:expr] = $val:expr) => {
        unsafe { *$slice.get_unchecked_mut($idx) = $val }
    };
}

fn levenshtein_distance<'w>(mut word_a: &'w str, mut word_b: &'w str) -> (usize, usize) {
    let mut m = word_a.chars().count();
    let mut n = word_b.chars().count();

    if m > n {
        mem::swap(&mut word_a, &mut word_b);
        mem::swap(&mut m, &mut n);
    }

    // u16 is sufficient considering the max length
    // of discord messages is smaller than u16::MAX
    let mut costs: Vec<_> = (0..=n as u16).collect();

    // SAFETY for get! and set!:
    // chars(word_a) <= chars(word_b) = N < N + 1 = costs.len()

    for (a, i) in word_a.chars().zip(1..) {
        let mut last_val = i;

        for (b, j) in word_b.chars().zip(1..) {
            let new_val = if a == b {
                get!(costs[j - 1])
            } else {
                get!(costs[j - 1]).min(last_val).min(get!(costs[j])) + 1
            };

            set!(costs[j - 1] = last_val);
            last_val = new_val;
        }

        set!(costs[n] = last_val);
    }

    (get!(costs[n]) as usize, n)
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
    let file = fs::read_to_string("../danser/danser.log")
        .await
        .context("failed to read danser logs")?;

    let line = file
        .lines()
        .find(|line| line.contains("Playing:"))
        .ok_or_else(|| anyhow!("expected `Playing:` in danser logs"))?;

    let map_without_artist = line
        .splitn(4, ' ')
        .last()
        .ok_or_else(|| anyhow!("expected at least 5 words in danser log line `{}`", line))?;

    Ok(map_without_artist.to_string())
}

fn check_server_prefix(server_prefixes: Vec<String>, params: &str) -> bool {
    server_prefixes
        .iter()
        .any(|p| params.starts_with(p) && params[p.len()..].starts_with("start"))
}
