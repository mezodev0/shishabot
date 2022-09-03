use std::sync::Arc;

use command_macros::SlashCommand;
use eyre::{Context as _, Result};
use osu_db::{Mode, Replay};
use tokio::{fs::File, io::AsyncWriteExt};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::channel::Attachment;

use crate::{
    core::{BotConfig, Context, ReplayData, TimePoints},
    util::{
        builder::MessageBuilder, interaction::InteractionCommand, Authored, InteractionCommandExt,
    },
};

#[derive(CreateCommand, CommandModel, SlashCommand)]
#[command(name = "render")]
/// Render a replay file and upload it
pub struct Render {
    #[command(name = "replay")]
    /// Specify the replay through a .osr file
    attachment: Attachment,
    #[command(min_value = 0, max_value = 65_535)]
    /// Specify a start timestamp in seconds
    start: Option<u16>,
    #[command(min_value = 0, max_value = 65_535)]
    /// Specify an end timestamp in seconds
    end: Option<u16>,
}

pub async fn slash_render(ctx: Arc<Context>, mut command: InteractionCommand) -> Result<()> {
    let Render {
        attachment,
        start,
        end,
    } = Render::from_interaction(command.input_data())?;

    if !matches!(attachment.filename.split('.').last(), Some("osr")) {
        let content = "The attachment must be a .osr file!";
        command.error(&ctx, content).await?;

        return Ok(());
    }

    let output_channel = match command.guild_id {
        Some(guild) => match ctx.guild_settings(guild, |server| server.output_channel) {
            Some(Some(output_channel)) => output_channel,
            Some(None) => {
                let content = "Looks like this server has not setup their output channel yet.\n\
                    Be sure to use `/setup` first.";
                command.error(&ctx, content).await?;

                return Ok(());
            }
            None => {
                let content = "Looks like this server has not setup their output channel yet.\n\
                    Be sure to use `/setup` first.";
                command.error(&ctx, content).await?;

                return Ok(());
            }
        },
        None => command.channel_id,
    };

    let bytes = match ctx.client().get_discord_attachment(&attachment).await {
        Ok(bytes) => bytes,
        Err(err) => {
            command.error(&ctx, "Failed to download attachment").await?;

            return Err(err);
        }
    };

    let replay = match Replay::from_bytes(&bytes) {
        Ok(replay) => replay,
        Err(err) => {
            let content = "Failed to parse the .osr file. Did you give a valid replay file?";
            command.error(&ctx, content).await?;

            return Err(err).context("failed to parse .osr file");
        }
    };

    if replay.mode != Mode::Standard {
        let content = "danser only accepts osu!standard plays, sorry :(";
        command.error(&ctx, content).await?;

        return Ok(());
    }

    let config = BotConfig::get();
    let mut replay_file = config.paths.downloads();
    replay_file.push(attachment.filename);

    let mut file = match File::create(&replay_file).await {
        Ok(file) => file,
        Err(err) => {
            command.error(&ctx, "Failed to store replay file").await?;

            return Err(err).with_context(|| format!("failed to create file `{replay_file:?}`"));
        }
    };

    if let Err(err) = file.write_all(&bytes).await {
        command.error(&ctx, "Failed to store replay file").await?;

        return Err(err).with_context(|| format!("failed writing to file `{replay_file:?}`"));
    };

    let replay_data = ReplayData {
        input_channel: command.channel_id,
        output_channel,
        path: replay_file,
        replay: replay.into(),
        time_points: TimePoints { start, end },
        user: command.user_id()?,
    };

    ctx.replay_queue.push(replay_data).await;

    let content = "Replay has been pushed to the queue!";
    let builder = MessageBuilder::new().embed(content);

    command.update(&ctx, &builder).await?;

    Ok(())
}
