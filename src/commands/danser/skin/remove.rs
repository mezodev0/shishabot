use std::{fs, sync::Arc};

use eyre::{Context as _, Result};

use crate::{
    core::{BotConfig, Context},
    util::{builder::MessageBuilder, interaction::InteractionCommand, InteractionCommandExt},
};

use super::SkinRemove;

pub async fn remove(
    ctx: Arc<Context>,
    command: InteractionCommand,
    args: SkinRemove,
) -> Result<()> {
    let SkinRemove { index } = args;

    let skin_path = BotConfig::get().paths.skins();
    let skin_dir = fs::read_dir(&skin_path).context("failed to read skins folder")?;

    let mut skin_list = skin_dir
        .collect::<Result<Vec<_>, _>>()
        .context("failed to read entry in skins folder")?;

    skin_list.sort_unstable_by_key(|entry| entry.file_name().to_ascii_lowercase());

    if let Some(skin_to_remove) = skin_list.get(index - 1) {
        fs::remove_dir_all(skin_to_remove.path())?;

        // Reset the skin list cache
        ctx.skin_list().clear();

        let skin = skin_to_remove.file_name();
        let content = format!("Successfully deleted skin `{}`", skin.to_string_lossy());
        let builder = MessageBuilder::new().embed(content);

        command.callback(&ctx, builder, false).await?;
    } else {
        let len = skin_list.len();
        let content = format!("Invalid skin index, must be between 1 and {len}");
        command.error_callback(&ctx, content, false).await?;
    }

    Ok(())
}
