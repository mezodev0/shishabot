use std::{fs, sync::Arc};

use eyre::{Report, Result};

use crate::{
    core::{BotConfig, Context},
    util::{
        builder::MessageBuilder, interaction::InteractionCommand, Authored, InteractionCommandExt,
    },
};

use super::{SettingsConfirm, SettingsDefault};

pub async fn default(
    ctx: Arc<Context>,
    command: InteractionCommand,
    args: SettingsDefault,
) -> Result<()> {
    if matches!(args.confirm, SettingsConfirm::Cancel) {
        let content = "Restoring default settings was cancelled.\n\
            Be sure to confirm if you want to overwrite your current settings with default values.";

        let builder = MessageBuilder::new().embed(content);
        command.callback(&ctx, builder, true).await?;

        return Ok(());
    }

    let author = command.user_id()?;
    let danser_path = BotConfig::get().paths.danser();

    let mut from = danser_path.to_owned();
    from.push("settings/default.json");

    if !from.exists() {
        let content = "Could not find default settings";
        let _ = command.error_callback(&ctx, content, false).await;

        bail!("No default settings found at {from:?}");
    }

    let mut to = danser_path.to_owned();
    to.push(format!("settings/{author}.json"));

    if let Err(err) = fs::copy(from, to) {
        let content = "Failed to set default values";
        let _ = command.error_callback(&ctx, content, false).await;

        let err =
            Report::from(err).wrap_err(format!("failed to copy default settings for {author}"));

        Err(err)
    } else {
        let content = "Successfully restored default settings!";
        let builder = MessageBuilder::new().embed(content);
        command.callback(&ctx, builder, false).await?;

        Ok(())
    }
}
