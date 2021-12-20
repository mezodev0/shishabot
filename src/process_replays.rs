use osu_db::Replay;
use reqwest::{
    multipart::{Form, Part},
    Client, Error,
};
use rosu_pp::{Beatmap, BeatmapExt};
use rosu_v2::prelude::{Beatmapset, GameMode, GameMods, Osu};
use serde::Deserialize;
use serenity::{
    http::Http,
    model::{
        channel::Message,
        id::{ChannelId, UserId},
    },
};
use std::{env, io::Cursor, mem, path::Path, sync::Arc, time::Duration};
use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    time,
};
use zip::ZipArchive;

#[derive(Deserialize)]
pub struct UploadResponse {
    pub shortcode: String,
    pub status: i8,
}

pub enum AttachmentParseResult {
    NoAttachmentOrReplay,
    BeingProcessed,
    FailedDownload(serenity::prelude::SerenityError),
    FailedParsing(osu_db::Error),
}

pub struct Data {
    path: String,
    replay: Replay,
    channel: ChannelId,
    user: UserId,
}

pub async fn process_replay(mut receiver: UnboundedReceiver<Data>, osu: Osu, http: Arc<Http>) {
    while let Some(replay_data) = receiver.recv().await {
        let replay_path = replay_data.path;
        let replay_file = replay_data.replay;
        let replay_user = replay_data.user;
        let replay_channel = replay_data.channel;

        let mapset = match replay_file.beatmap_hash.as_deref() {
            Some(hash) => match osu.beatmap().checksum(hash).await {
                Ok(map) => map.mapset.unwrap(),
                Err(why) => {
                    println!("failed to request map with hash {}: {}", hash, why);
                    continue;
                }
            },
            None => {
                println!("no hash in replay requested by user {}", replay_user);
                continue;
            }
        };

        let mapset_id = mapset.mapset_id;

        download_mapset(mapset_id).await;

        let settings = if path_exists(format!("../danser/settings/{}.json", replay_user)).await {
            format!("{}", replay_user)
        } else {
            "default".to_string()
        };

        let filename = replay_path
            .split('/')
            .last()
            .and_then(|file| file.split('.').next())
            .unwrap();

        let mut command = Command::new("../danser/danser");

        command
            .arg(format!("-replay={}", replay_path))
            .arg("-record")
            .arg(format!("-settings={}", settings))
            .arg("-quickstart")
            .arg(format!("-out={}", filename));

        let output = command.output().await.unwrap();
        let stdout = std::str::from_utf8(&output.stdout).unwrap();
        let stderr = std::str::from_utf8(&output.stderr).unwrap();

        println!("stdout: {}", stdout);
        println!("stderr: {}", stderr);

        let map_path = format!(
            "../Songs/{}/{}",
            mapset_id,
            get_beatmap_osu_file(mapset_id).await
        );

        let streamable_title = create_title(&replay_file, map_path, &mapset).await;

        let shortcode: UploadResponse =
            match upload(format!("../Replays/{}.mp4", filename), streamable_title).await {
                Ok(json) => json,
                Err(why) => panic!("failed to upload file: {}", why),
            };

        time::sleep(Duration::from_secs(5)).await;

        let content = format!(
            "<@{}> your replay is ready! https://streamable.com/{}",
            replay_user, shortcode.shortcode
        );

        if let Err(why) = replay_channel
            .send_message(&http, |m| m.content(content))
            .await
        {
            println!("couldnt send streamable link: {}", why);
        }
    }
}

pub async fn parse_attachment_replay(
    msg: &Message,
    sender: &UnboundedSender<Data>,
) -> AttachmentParseResult {
    let attachment = match msg.attachments.last() {
        Some(a) if matches!(a.filename.split('.').last(), Some("osr")) => a,
        Some(_) | None => return AttachmentParseResult::NoAttachmentOrReplay,
    };

    match attachment.download().await {
        // parse the data as replay
        Ok(bytes) => match osu_db::Replay::from_bytes(&bytes) {
            Ok(replay) => {
                let mut file = File::create(format!("../Downloads/{}", &attachment.filename))
                    .await
                    .expect("failed to create file");

                file.write_all(&bytes).await.expect("failed to write");

                let replay_data = Data {
                    path: format!("../Downloads/{}", &attachment.filename),
                    replay,
                    channel: msg.channel_id,
                    user: msg.author.id,
                };

                if let Err(why) = sender.send(replay_data) {
                    println!("failed to send: {}", why);
                }

                AttachmentParseResult::BeingProcessed
            }
            Err(why) => AttachmentParseResult::FailedParsing(why),
        },
        Err(why) => AttachmentParseResult::FailedDownload(why),
    }
}

pub async fn upload(filepath: String, title: String) -> Result<UploadResponse, Error> {
    let username = env::var("STREAMABLE_USERNAME").unwrap();
    let password = env::var("STREAMABLE_PASSWORD").unwrap();

    let endpoint = "https://api.streamable.com/upload";

    let form = Form::new()
        .part("file", file(filepath).await.unwrap())
        .text("title", title);

    let client = Client::new();
    let resp = client
        .post(endpoint)
        .basic_auth(username, Some(password))
        .multipart(form)
        .send()
        .await
        .unwrap();

    let response_as_json = resp.json::<UploadResponse>().await.unwrap();
    Ok(response_as_json)
}

async fn path_exists(path: String) -> bool {
    fs::metadata(path).await.is_ok()
}

pub async fn file<T: AsRef<Path>>(path: T) -> Result<Part, tokio::io::Error> {
    let path = path.as_ref();
    let file_name = path
        .file_name()
        .map(|filename| filename.to_string_lossy().into_owned());
    let ext = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    let mime = mime_guess::from_ext(ext).first_or_octet_stream();
    let mut file = File::open(path).await?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).await?;
    let field = Part::bytes(bytes).mime_str(mime.essence_str()).unwrap();

    Ok(if let Some(file_name) = file_name {
        field.file_name(file_name)
    } else {
        field
    })
}

async fn download_mapset(mapset_id: u32) {
    let url = format!("https://kitsu.moe/d/{}", mapset_id);
    let bytes = reqwest::get(url).await.unwrap().bytes().await.unwrap();
    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor).unwrap();
    let out_path = format!("../Songs/{}", mapset_id);
    archive.extract(out_path).unwrap();
}

async fn create_title(replay: &Replay, map_path: String, mapset: &Beatmapset) -> String {
    let mods = replay.mods.bits();

    let difficulty = match Beatmap::from_path(&map_path) {
        Ok(map) => map.stars(mods, None),
        Err(why) => panic!("Error while parsing map: {}, path: {}", why, map_path),
    };

    let mods_str = GameMods::from_bits(mods).unwrap().to_string();
    let stars = (difficulty.stars() * 100.0).round() / 100.0;
    let player = replay.player_name.as_ref().unwrap();
    let map_title = &mapset.title;
    let acc = accuracy(replay, GameMode::STD);

    let title = format!(
        "[{}⭐] {} | {} +{} {}%",
        stars, player, map_title, mods_str, acc
    );

    title
}

async fn get_beatmap_osu_file(mapset_id: u32) -> String {
    let file: String = match fs::read_to_string("../danser/danser.log").await {
        Ok(log) => log,
        Err(why) => panic!("failed to read file: {}", why),
    };

    let line = file.lines().find(|line| line.contains("Playing:")).unwrap();

    let map_without_artist = line.splitn(4, ' ').last().unwrap().to_string();

    let items = std::fs::read_dir(format!("../Songs/{}", mapset_id)).unwrap();

    let mut max_similarity: f32 = 0.0;
    let mut final_file_name = String::new();

    for item in items {
        let file_name = item.unwrap().file_name();
        let item_file_name = file_name.to_string_lossy();

        println!("COMPARING: {} WITH: {}", map_without_artist, item_file_name);

        let similarity = levenshtein_similarity(&map_without_artist, &item_file_name);

        if similarity > max_similarity {
            max_similarity = similarity;
            final_file_name = item_file_name.into_owned();
        }
    }

    println!(
        "FINAL TITLE: {} SIMILARITY: {}",
        final_file_name, max_similarity
    );

    final_file_name
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
