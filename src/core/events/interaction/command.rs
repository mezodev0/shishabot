use std::{mem, sync::Arc};

use eyre::{Context as _, Result};

use crate::{
    core::{
        commands::{
            checks::check_authority,
            slash::{SlashCommand, SlashCommands},
        },
        events::{EventLocation, ProcessResult},
        BotConfig, Context,
    },
    util::{interaction::InteractionCommand, Authored, InteractionCommandExt},
};

pub async fn handle_command(ctx: Arc<Context>, mut command: InteractionCommand) {
    let name = mem::take(&mut command.data.name);

    {
        let username = command
            .user()
            .map(|u| u.name.as_str())
            .unwrap_or("<unknown user>");

        let location = EventLocation::new(&ctx, &command);
        info!("[{location}] {username} used command `{name}`");
    }

    let slash = match SlashCommands::get().command(&name) {
        Some(slash) => slash,
        None => return error!("unknown slash command `{name}`"),
    };

    let res = process_command(ctx, command, slash)
        .await
        .with_context(|| format!("failed to process slash command `{name}`"));

    match res {
        Ok(ProcessResult::Success) => info!("Processed slash command `{name}`"),
        Ok(res) => info!("Command `/{name}` was not processed: {res:?}"),
        Err(err) => error!("{err:?}"),
    }
}

async fn process_command(
    ctx: Arc<Context>,
    command: InteractionCommand,
    slash: &SlashCommand,
) -> Result<ProcessResult> {
    match pre_process_command(&ctx, &command, slash).await? {
        Some(result) => Ok(result),
        None => {
            if slash.flags.defer() {
                command.defer(&ctx, slash.flags.ephemeral()).await?;
            }

            (slash.exec)(ctx, command).await?;

            Ok(ProcessResult::Success)
        }
    }
}

async fn pre_process_command(
    ctx: &Context,
    command: &InteractionCommand,
    slash: &SlashCommand,
) -> Result<Option<ProcessResult>> {
    let user_id = command.user_id()?;

    // Only for owners?
    if slash.flags.only_owner() && !BotConfig::get().owners.contains(&user_id) {
        let content = "That command can only be used by the bot owner";
        command.error_callback(ctx, content, false).await?;

        return Ok(Some(ProcessResult::NoOwner));
    }

    // Only for authorities?
    if slash.flags.authority() {
        match check_authority(ctx, user_id, command.channel_id, command.guild_id).await {
            None => {}
            Some(content) => {
                command.error_callback(ctx, content, false).await?;

                return Ok(Some(ProcessResult::NoAuthority));
            }
        }
    }

    Ok(None)
}
