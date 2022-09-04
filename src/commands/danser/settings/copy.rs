use std::{fs, sync::Arc};

use eyre::{Report, Result};

use crate::{
    core::{BotConfig, Context},
    util::{
        builder::MessageBuilder, interaction::InteractionCommand, Authored, InteractionCommandExt,
    },
};

use super::SettingsCopy;

pub async fn copy(
    ctx: Arc<Context>,
    command: InteractionCommand,
    args: SettingsCopy,
) -> Result<()> {
    let author = command.user_id()?;
    let SettingsCopy { user } = args;

    if author == user {
        command.error_callback(&ctx, ":clown:", false).await?;

        return Ok(());
    }

    let danser_path = BotConfig::get().paths.danser();

    let mut from = danser_path.to_owned();
    from.push(format!("settings/{user}.json"));

    if !from.exists() {
        let content = "That user has not configured their settings yet";
        command.error_callback(&ctx, content, false).await?;

        return Ok(());
    }

    let mut to = danser_path.to_owned();
    to.push(format!("settings/{author}.json"));

    if let Err(err) = fs::copy(from, to) {
        let content = "Failed to copy the settings file";
        let _ = command.error_callback(&ctx, content, false).await;

        let err =
            Report::from(err).wrap_err(format!("failed to copy settings from {user} for {author}"));

        Err(err)
    } else {
        let content = "Successfully copied settings!";
        let builder = MessageBuilder::new().embed(content);
        command.callback(&ctx, builder, false).await?;

        Ok(())
    }
}
