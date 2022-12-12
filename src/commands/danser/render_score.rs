use std::{fs, sync::Arc};

use command_macros::msg_command;
use eyre::{Context as _, ContextCompat, Report};
use osu_db::Replay;
use rosu_v2::prelude::Score;
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time};
use twilight_interactions::command::CommandInputData;
use twilight_model::{channel::embed::Embed, util::Timestamp};

use crate::{
    core::{replay_queue::ReplaySlim, BotConfig, Context, ReplayData, TimePoints},
    util::{
        builder::MessageBuilder, interaction::InteractionCommand, Authored, InteractionCommandExt,
    },
};

#[msg_command(name = "Render score", dm_permission = false)]
async fn render_from_msg(ctx: Arc<Context>, mut command: InteractionCommand) -> Result<()> {
    let input_data = command.input_data();

    let (osu_user_id, timestamp) = match parse_embed(&input_data) {
        Some(ParsedEmbed { user_id, timestamp }) => (user_id, timestamp),
        None => {
            let content = "The command can only be used on Bathbot **/rs** embeds!";
            command.error(&ctx, content).await?;

            return Ok(());
        }
    };

    let ts_unix = OffsetDateTime::from_unix_timestamp(timestamp.as_secs())
        .unwrap()
        .unix_timestamp();

    // check recents
    let recent_scores = ctx
        .osu()
        .user_scores(osu_user_id)
        .recent()
        .include_fails(false)
        .limit(100)
        .await
        .context("failed to get recent scores")?;

    let score_to_render = recent_scores.into_iter().find(|score| {
        (score.ended_at.unix_timestamp() - ts_unix).abs() <= 3 && score.replay.unwrap_or(false)
    });

    // check tops
    let score_to_render = match score_to_render {
        Some(score) => score,
        None => {
            let top_scores = ctx
                .osu()
                .user_scores(osu_user_id)
                .best()
                .limit(100)
                .await
                .context("failed to get top scores")?;

            let score_opt = top_scores.into_iter().find(|score| {
                (score.ended_at.unix_timestamp() - ts_unix).abs() <= 3
                    && score.replay.unwrap_or(false)
            });

            match score_opt {
                Some(score) => score,
                None => {
                    let content = "Couldn't find the replay for this score";
                    command.error(&ctx, content).await?;

                    return Ok(());
                }
            }
        }
    };

    let score_id = match score_to_render.score_id {
        Some(id) => id,
        None => {
            let content = "Couldn't find the ID for this score";
            command.error(&ctx, content).await?;

            return Ok(());
        }
    };

    let mut replay_bytes = ctx
        .client()
        .get_raw_replay(score_id)
        .await
        .context("failed to get replay bytes")?;

    extend_replay_bytes(&mut replay_bytes, &score_to_render);

    let replay_user = score_to_render.user;

    let osu_user = match replay_user {
        Some(replay_user) => replay_user.username,
        None => "<unknown user>".into(),
    };

    let replay_mapset = score_to_render.mapset;

    let map_title = match replay_mapset {
        Some(replay_mapset) => replay_mapset.title,
        None => "<unknown map>".into(),
    };

    let mut path = BotConfig::get().paths.downloads();
    path.push(format!("{osu_user} - {map_title}.osr"));

    fs::write(&path, &replay_bytes).context("failed to write into replay file")?;

    let replay = match Replay::from_bytes(&replay_bytes) {
        Ok(replay) => ReplaySlim::from(replay),
        Err(err) => {
            let content = "Failed to parse replay";
            let _ = command.error(&ctx, content).await;

            return Err(Report::new(err).wrap_err("failed to parse replay"));
        }
    };

    let input_channel = command.channel_id;
    let user = command.user_id()?;

    let guild_id = command.guild_id().context("expected guild id")?;
    let output_channel = ctx
        .guild_settings(guild_id, |server| server.output_channel)
        .flatten()
        .unwrap_or(input_channel);

    let replay_data = ReplayData {
        input_channel,
        output_channel,
        pitch: None,
        path,
        replay,
        user,
        time_points: TimePoints { start: 0, end: 0 },
    };

    ctx.replay_queue.push(replay_data).await;

    let builder = MessageBuilder::new().embed("Replay has been pushed to the queue!");
    command.update(&ctx, &builder).await?;

    Ok(())
}

struct ParsedEmbed {
    user_id: u32,
    timestamp: Timestamp,
}

fn parse_embed(input_data: &CommandInputData<'_>) -> Option<ParsedEmbed> {
    let embed = input_data
        .resolved
        .as_ref()
        .and_then(|resolved| resolved.messages.values().next())
        .and_then(|msg| msg.embeds.first())?;

    let user_url = embed.author.as_ref().and_then(|a| a.url.as_ref())?;

    let user_id = user_url
        .split('/')
        .nth(4)
        .and_then(|id| id.parse::<u32>().ok())?;

    let timestamp = embed
        .timestamp
        .or_else(|| get_timestamp_from_minimized_embed(embed))?;

    Some(ParsedEmbed { user_id, timestamp })
}

fn get_timestamp_from_minimized_embed(embed: &Embed) -> Option<Timestamp> {
    let field = embed.fields.first()?;

    let discord_timestamp = field.name.rsplit('\t').next()?;

    let actual_timestamp_value = discord_timestamp
        .trim_start_matches("<t:")
        .trim_end_matches(":R>");

    let timestamp_value_as_int = actual_timestamp_value.parse().ok()?;

    Timestamp::from_secs(timestamp_value_as_int).ok()
}

// https://osu.ppy.sh/wiki/en/Client/File_formats/Osr_%28file_format%29
fn extend_replay_bytes(bytes: &mut Vec<u8>, score: &Score) {
    let initial_len = bytes.len();
    let mut bytes_written = 0;

    bytes_written += encode_byte(bytes, score.mode as u8);
    bytes_written += encode_int(bytes, game_version(score.ended_at.date()));

    let map_md5 = score
        .map
        .as_ref()
        .and_then(|map| map.checksum.as_deref())
        .unwrap_or_default();

    bytes_written += encode_string(bytes, map_md5);

    let username = score
        .user
        .as_ref()
        .map(|user| user.username.as_str())
        .unwrap_or_default();

    bytes_written += encode_string(bytes, username);

    let replay_md5 = String::new();
    bytes_written += encode_string(bytes, &replay_md5);

    let stats = &score.statistics;
    bytes_written += encode_short(bytes, stats.count_300 as u16);
    bytes_written += encode_short(bytes, stats.count_100 as u16);
    bytes_written += encode_short(bytes, stats.count_50 as u16);
    bytes_written += encode_short(bytes, stats.count_geki as u16);
    bytes_written += encode_short(bytes, stats.count_katu as u16);
    bytes_written += encode_short(bytes, stats.count_miss as u16);

    bytes_written += encode_int(bytes, score.score);

    bytes_written += encode_short(bytes, score.max_combo as u16);

    bytes_written += encode_byte(bytes, score.perfect as u8);

    bytes_written += encode_int(bytes, score.mods.bits());

    let lifebar = String::new();
    bytes_written += encode_string(bytes, &lifebar);

    bytes_written += encode_datetime(bytes, score.ended_at);

    bytes_written += encode_int(bytes, initial_len as u32);

    bytes.rotate_right(bytes_written);

    encode_long(bytes, score.score_id.unwrap_or(0));
}

fn encode_byte(bytes: &mut Vec<u8>, byte: u8) -> usize {
    bytes.push(byte);

    1
}

fn encode_short(bytes: &mut Vec<u8>, short: u16) -> usize {
    bytes.extend_from_slice(&short.to_le_bytes());

    2
}

fn encode_int(bytes: &mut Vec<u8>, int: u32) -> usize {
    bytes.extend_from_slice(&int.to_le_bytes());

    4
}

fn encode_long(bytes: &mut Vec<u8>, long: u64) -> usize {
    bytes.extend_from_slice(&long.to_le_bytes());

    8
}

fn encode_string(bytes: &mut Vec<u8>, s: &str) -> usize {
    if s.is_empty() {
        bytes.push(0x00); // "no string"

        1
    } else {
        bytes.push(0x0b); // "string incoming"
        let len = encode_leb128(bytes, s.len());
        bytes.extend_from_slice(s.as_bytes());

        1 + len + s.len()
    }
}

// https://en.wikipedia.org/wiki/LEB128
fn encode_leb128(bytes: &mut Vec<u8>, mut n: usize) -> usize {
    let mut bytes_written = 0;

    loop {
        let mut byte = ((n & u8::MAX as usize) as u8) & !(1 << 7);
        n >>= 7;

        if n != 0 {
            byte |= 1 << 7;
        }

        bytes.push(byte);
        bytes_written += 1;

        if n == 0 {
            return bytes_written;
        }
    }
}

// https://docs.microsoft.com/en-us/dotnet/api/system.datetime.ticks?redirectedfrom=MSDN&view=net-6.0#System_DateTime_Ticks
fn encode_datetime(bytes: &mut Vec<u8>, datetime: OffsetDateTime) -> usize {
    let orig_date = Date::from_ordinal_date(1, 1).unwrap();
    let orig_time = Time::from_hms(0, 0, 0).unwrap();

    let orig = PrimitiveDateTime::new(orig_date, orig_time).assume_utc();

    let orig_nanos = orig.unix_timestamp_nanos();
    let this_nanos = datetime.unix_timestamp_nanos();

    let long = (this_nanos - orig_nanos) / 100;

    encode_long(bytes, long as u64)
}

fn game_version(date: Date) -> u32 {
    let mut version = date.year() as u32;
    version *= 100;

    version += date.month() as u32;
    version *= 100;

    version += date.day() as u32;

    version
}
