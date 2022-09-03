use std::{
    error::Error as StdError,
    ffi::OsStr,
    fmt::{Display, Formatter, Result as FmtResult},
    fs,
    io::Cursor,
    path::PathBuf,
    sync::Arc,
};

use bytes::Bytes;
use eyre::{Context as _, ContextCompat, Report, Result};
use rosu_pp::{Beatmap, BeatmapExt};
use rosu_v2::prelude::{Beatmap as Map, GameMods};
use tokio::process::Command;
use zip::ZipArchive;

use crate::{
    core::{BotConfig, Context, ReplayStatus},
    util::{builder::MessageBuilder, levenshtein_similarity, ChannelExt},
};

use super::{ReplayData, ReplayQueue, ReplaySlim};

impl ReplayQueue {
    pub fn process(ctx: Arc<Context>) {
        tokio::spawn(Self::async_process(ctx));
    }

    async fn async_process(ctx: Arc<Context>) {
        let config = BotConfig::get();

        let mut danser_path = config.paths.danser().to_owned();
        danser_path.push("danser");

        loop {
            let ReplayData {
                input_channel,
                output_channel,
                path,
                replay,
                time_points,
                user,
            } = ctx.replay_queue.peek().await;

            let mapset_id = match replay.beatmap_hash.as_deref() {
                Some(hash) => match ctx.osu().beatmap().checksum(hash).await {
                    Ok(Map { mapset, .. }) => match mapset {
                        Some(mapset) => mapset.mapset_id,
                        None => {
                            warn!("map without mapset");

                            let content = "The mapset was not received when requesting the map from the osu!api";
                            let _ = input_channel.error(&ctx, content).await;

                            ctx.replay_queue.reset_peek().await;
                            continue;
                        }
                    },
                    Err(err) => {
                        let context = format!("failed to request map with hash `{hash}`");
                        let err = Report::from(err).wrap_err(context);
                        warn!("{err:?}");

                        let content = "Failed to retrieve map. Maybe it's not submitted?";
                        let _ = input_channel.error(&ctx, content).await;

                        ctx.replay_queue.reset_peek().await;
                        continue;
                    }
                },
                None => {
                    warn!("missing hash in replay requested by user {user}");

                    let content = "Missing the beatmap hash in the replay file";
                    let _ = input_channel.error(&ctx, content).await;

                    ctx.replay_queue.reset_peek().await;
                    continue;
                }
            };

            info!("Started map download");
            ctx.replay_queue.set_status(ReplayStatus::Downloading).await;

            if let Err(err) = download_mapset(&ctx, mapset_id).await {
                warn!("{err:?}");

                let content = "Failed to download map. Mirrors are likely down, try again later.";
                let _ = input_channel.error(&ctx, content).await;

                ctx.replay_queue.reset_peek().await;
                continue;
            }

            info!("Finished map download");

            let mut settings_path = config.paths.danser().to_owned();
            settings_path.push(format!("settings/{user}.json"));

            let settings = if settings_path.exists() {
                user.to_string()
            } else {
                "default".to_owned()
            };

            let filename_opt = path
                .file_name()
                .and_then(OsStr::to_str)
                .and_then(|name| name.split('.').next());

            let filename = match filename_opt {
                Some(name) => name,
                None => {
                    warn!("replay path `{path:?}` has an unexpected form");

                    let content = "There was an error resolving the beatmap path";
                    let _ = input_channel.error(&ctx, content).await;

                    ctx.replay_queue.reset_peek().await;
                    continue;
                }
            };

            let mut command = Command::new(&danser_path);

            command
                .arg("-noupdatecheck")
                .arg("-replay")
                .arg(&path)
                .arg("-record")
                .arg("-settings")
                .arg(settings)
                .arg("-quickstart")
                .arg("-out")
                .arg(filename);

            if let Some(start) = time_points.start {
                command.args(["-start", &start.to_string()]);
            }

            if let Some(end) = time_points.end {
                command.args(["-end", &end.to_string()]);
            }

            info!("Started replay parsing");
            ctx.replay_queue.set_status(ReplayStatus::Processing).await;

            match command.output().await {
                Ok(output) => {
                    if let Ok(stdout) = std::str::from_utf8(&output.stdout) {
                        debug!("stdout: {}", stdout);
                    }

                    if let Ok(stderr) = std::str::from_utf8(&output.stderr) {
                        debug!("stderr: {}", stderr);
                    }
                }
                Err(err) => {
                    let err = Report::from(err).wrap_err("failed to get command output");
                    warn!("{err:?}");

                    let content = "Failed to parse replay";
                    let _ = input_channel.error(&ctx, content).await;

                    ctx.replay_queue.reset_peek().await;
                    continue;
                }
            }

            info!("Finished replay parsing");

            let title = match get_title() {
                Ok(title) => title,
                Err(err) => {
                    warn!("{err:?}");

                    let content = "Failed to read danser logs";
                    let _ = input_channel.error(&ctx, content).await;

                    ctx.replay_queue.reset_peek().await;
                    continue;
                }
            };

            let map_osu_file = match get_beatmap_osu_file(mapset_id, &title).await {
                Ok(osu_file) => osu_file,
                Err(err) => {
                    let err = err.wrap_err("failed to get map_osu_file");
                    warn!("{err:?}");

                    let content = "danser did not like the replay file";
                    let _ = input_channel.error(&ctx, content).await;

                    ctx.replay_queue.reset_peek().await;
                    continue;
                }
            };

            let mut map_path = config.paths.songs();
            map_path.push(format!("{mapset_id}/{map_osu_file}"));

            let video_title = match create_title(&replay, map_path, &title).await {
                Ok(title) => title,
                Err(err) => {
                    let err = err.wrap_err("failed to create title");
                    warn!("{err:?}");

                    let content = "There was an error while trying to create the video title";
                    let _ = input_channel.error(&ctx, content).await;

                    ctx.replay_queue.reset_peek().await;
                    continue;
                }
            };

            let mut file_path = config.paths.replays();
            file_path.push(format!("{filename}.mp4"));

            info!("Started upload to shisha.mezo.xyz");
            ctx.replay_queue.set_status(ReplayStatus::Uploading).await;

            let link = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";

            // TODO
            // let link = match uploader
            //     .upload_video(video_title, replay_user, &filepath)
            //     .await
            // {
            //     Ok(response) => {
            //         if response.error == 1 {
            //             warn!("failed to upload: {}", response.text);
            //             send_error_message(
            //                 &http,
            //                 input_channel,
            //                 replay_user,
            //                 format!("failed to upload: `{}`", response.text).as_str(),
            //             )
            //             .await;

            //             queue.reset_peek().await;
            //             continue;
            //         } else {
            //             response.text
            //         }
            //     }
            //     Err(why) => {
            //         warn!("{:?}", why.context("failed to upload file"));

            //         send_error_message(
            //             &http,
            //             input_channel,
            //             replay_user,
            //             "failed to upload to custom uploader",
            //         )
            //         .await;

            //         queue.reset_peek().await;
            //         continue;
            //     }
            // };

            info!("Finished upload to shisha.mezo.xyz");

            let content = format!("<@{user}> your replay is ready! {link}");
            let builder = MessageBuilder::new().content(content);

            if let Err(err) = output_channel.create_message(&ctx, &builder).await {
                let err = Report::from(err).wrap_err("failed to send video link");
                warn!("{err:?}");
            }

            ctx.replay_queue.reset_peek().await;
        }
    }
}

#[derive(Debug)]
struct MapsetDownloadError {
    kitsu: Report,
    chimu: Report,
}

impl Display for MapsetDownloadError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "failed to download mapset:\n\
            kitsu: {:?}\n\
            chimu: {:?}",
            self.kitsu, self.chimu
        )
    }
}

impl StdError for MapsetDownloadError {
    #[inline]
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

async fn download_mapset(ctx: &Context, mapset_id: u32) -> Result<()> {
    let bytes = request_mapset(ctx, mapset_id).await?;
    let cursor = Cursor::new(bytes);

    let mut archive = ZipArchive::new(cursor).context("failed to create zip archive")?;

    let mut out_path = BotConfig::get().paths.songs();
    out_path.push(mapset_id.to_string());

    archive
        .extract(&out_path)
        .with_context(|| format!("failed to extract zip archive at `{out_path:?}`"))
}

async fn request_mapset(ctx: &Context, mapset_id: u32) -> Result<Bytes> {
    let kitsu = match ctx.client().download_kitsu_mapset(mapset_id).await {
        Ok(bytes) => return Ok(bytes),
        Err(err) => err,
    };

    let chimu = match ctx.client().download_chimu_mapset(mapset_id).await {
        Ok(bytes) => return Ok(bytes),
        Err(err) => err,
    };

    Err(Report::from(MapsetDownloadError { kitsu, chimu }))
}

async fn create_title(replay: &ReplaySlim, map_path: PathBuf, map_title: &str) -> Result<String> {
    let stars = Beatmap::from_path(&map_path)
        .await
        .with_context(|| format!("failed to parse map at {map_path:?}"))?
        .stars()
        .mods(replay.mods)
        .calculate()
        .stars();

    // let map_title = get_title()?;
    let stars = (stars * 100.0).round() / 100.0;
    let player = replay.player_name.as_deref().unwrap_or("<unknown player>");
    let acc = replay.accuracy();

    let mods = match GameMods::from_bits(replay.mods) {
        Some(GameMods::NoMod) | None => String::new(),
        Some(mods) => format!("+{mods} "),
    };

    Ok(format!("[{stars}⭐] {player} | {map_title} {mods}{acc}%"))
}

async fn get_beatmap_osu_file(mapset_id: u32, map_without_artist: &str) -> Result<String> {
    let mut items_dir = BotConfig::get().paths.songs();
    items_dir.push(mapset_id.to_string());

    let items = fs::read_dir(&items_dir)
        .with_context(|| format!("failed to read items dir at {items_dir:?}"))?;

    let mut max_similarity = 0.0;
    let mut final_file_name = String::new();

    for entry in items {
        match entry {
            Ok(entry) => {
                let file_name = entry.file_name();

                if let Some(file_name) = file_name.to_str().filter(|name| name.ends_with(".osu")) {
                    debug!("COMPARING: {map_without_artist} WITH: {file_name}");

                    let similarity = levenshtein_similarity(map_without_artist, file_name);

                    if similarity > max_similarity {
                        max_similarity = similarity;
                        final_file_name = file_name.to_owned();
                    }
                }
            }
            Err(err) => {
                let context = format!("there was an error while reading files in {items_dir:?}");

                return Err(Report::from(err).wrap_err(context));
            }
        }
    }

    debug!("FINAL TITLE: {final_file_name} SIMILARITY: {max_similarity}");

    Ok(final_file_name)
}

fn get_title() -> Result<String> {
    let mut logs_path = BotConfig::get().paths.danser().to_owned();
    logs_path.push("danser.log");

    let logs = fs::read_to_string(logs_path).context("failed to read danser logs")?;

    let line = logs
        .lines()
        .find(|line| line.contains("Playing:"))
        .context("expected `Playing:` in danser logs")?;

    line.splitn(4, ' ')
        .last()
        .map(str::to_owned)
        .with_context(|| format!("expected at least 5 words in danser log line `{line}`"))
}
