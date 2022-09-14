use std::{str::FromStr, sync::Arc};

use command_macros::msg_command;
use time::{format_description::well_known::Iso8601, OffsetDateTime};
use twilight_model::{channel::embed::Embed, util::Timestamp};

use crate::{
    core::{BotConfig, Context, ReplayData, TimePoints},
    util::{builder::MessageBuilder, interaction::InteractionCommand, InteractionCommandExt},
};

#[msg_command(name = "Render score", dm_permission = false)]
async fn render_from_msg(ctx: Arc<Context>, mut command: InteractionCommand) -> Result<()> {
    let resolved = match command.input_data().resolved {
        Some(resolved) => resolved,
        None => {
            command.error(&ctx, "?").await?;
            return Ok(());
        }
    };

    let message = match resolved.messages.values().next() {
        Some(message) => message,
        None => {
            command
                .error(
                    &ctx,
                    "The command can only be used on Bathbot **/rs** embeds!",
                )
                .await?;
            return Ok(());
        }
    };

    let embed = match message.embeds.first() {
        Some(embed) => embed,
        None => {
            command
                .error(
                    &ctx,
                    "The command can only be used on Bathbot **/rs** embeds!",
                )
                .await?;
            return Ok(());
        }
    };

    let osu_user_url = match embed.author.as_ref().and_then(|a| a.url.as_ref()) {
        Some(url) => url,
        None => {
            command
                .error(
                    &ctx,
                    "The command can only be used on Bathbot **/rs** embeds!",
                )
                .await?;
            return Ok(());
        }
    };

    let osu_user_id = match osu_user_url.split("/").nth(4) {
        Some(id) => match id.parse::<u32>() {
            Ok(id) => id,
            Err(_) => {
                command
                    .error(
                        &ctx,
                        "The command can only be used on Bathbot **/rs** embeds!",
                    )
                    .await?;
                return Ok(());
            }
        },
        None => {
            command
                .error(
                    &ctx,
                    "The command can only be used on Bathbot **/rs** embeds!",
                )
                .await?;
            return Ok(());
        }
    };

    let timestamp = match embed.timestamp {
        // Config set embeds to 'Always Maximized' or 'Initially Maximized'
        Some(timestamp) => timestamp,
        // Config set embeds to 'Always Minimized'
        None => match get_timestamp_from_minimized_embed(embed) {
            Some(timestamp) => timestamp,
            None => {
                command
                    .error(
                        &ctx,
                        "The command can only be used on Bathbot **/rs** embeds!",
                    )
                    .await?;
                return Ok(());
            }
        },
    };

    let mut score_to_render: Option<rosu_v2::prelude::Score> = None;
    let ts_unix = OffsetDateTime::from_unix_timestamp(timestamp.as_secs())
        .unwrap()
        .unix_timestamp();

    // check recents
    let recent_scores = ctx.osu().user_scores(osu_user_id).recent().await?;

    for score in recent_scores {
        let score_ts = score.ended_at.unix_timestamp();
        if (score_ts - ts_unix).abs() <= 3 && score.replay.unwrap_or(false) {
            score_to_render = Some(score);
            break;
        }
    }

    // check tops
    if score_to_render.is_none() {
        let top_scores = ctx.osu().user_scores(osu_user_id).best().await?;

        for score in top_scores {
            let score_ts = score.ended_at.unix_timestamp();
            if (score_ts - ts_unix).abs() <= 3 && score.replay.unwrap_or(false) {
                score_to_render = Some(score);
                break;
            }
        }

        if score_to_render.is_none() {
            command
                .error(&ctx, "Couldn't find the replay for this score")
                .await?;
            return Ok(());
        }
    }

    let score_id = score_to_render.unwrap().score_id;
    let replay = ctx.client().get_osu_replay(score_id).await?;

    let input_channel = command.channel_id;
    let user = command.member.as_ref().unwrap().user.as_ref().unwrap().id;
    let output_channel = ctx
        .http
        .create_private_channel(user)
        .exec()
        .await?
        .model()
        .await?
        .id;

    let mut path = BotConfig::get().paths.downloads();
    path.push(format!("{score_id}.osr"));

    let time_points = TimePoints { start: 0, end: 0 };

    let replay_data = ReplayData {
        input_channel,
        output_channel,
        path,
        replay,
        time_points,
        user,
    };

    ctx.replay_queue.push(replay_data).await;

    let builder = MessageBuilder::new().embed("Replay has been pushed to the queue!");
    command.update(&ctx, &builder).await?;

    Ok(())
}

fn get_timestamp_from_minimized_embed(embed: &Embed) -> Option<Timestamp> {
    let field = embed.fields.first()?;

    let discord_timestamp = field.name.split("\t").last()?;

    let actual_timestamp_value = discord_timestamp
        .trim_start_matches("<t:")
        .trim_end_matches(":R>");

    let timestamp_value_as_int = actual_timestamp_value.parse().ok()?;

    let datetime = match time::OffsetDateTime::from_unix_timestamp(timestamp_value_as_int) {
        Ok(datetime) => datetime,
        Err(_) => return None,
    };

    let datetime_formatted = match datetime.format(&Iso8601::DEFAULT) {
        Ok(datetime_formatted) => datetime_formatted,
        Err(_) => return None,
    };

    match Timestamp::from_str(&datetime_formatted) {
        Ok(timestamp) => return Some(timestamp),
        Err(_) => return None,
    };
}
