use eyre::Result;
use std::{fs::DirEntry, sync::Arc};
use tokio::fs;

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
    let config = BotConfig::get();
    let skin_path = config.paths.skins();

    let mut skin_dir = fs::read_dir(skin_path).await?;
    let mut skin_list = Vec::new();
    while let Some(skin) = skin_dir.next_entry().await? {
        skin_list.push(skin);
    }
    let skin_to_remove = skin_list.get((index - 1) as usize);

    if let Some(skin_to_remove) = skin_to_remove {
        fs::remove_dir_all(skin_to_remove.path()).await?;
        let content = format!(
            "Successfully deleted skin `{}`",
            skin_to_remove.file_name().to_string_lossy()
        );
        let builder = MessageBuilder::new().embed(content);
        command.callback(&ctx, builder, false).await?;
    } else {
        command
            .error_callback(&ctx, "The skin you wanted to remove does not exist!", false)
            .await?;
    }

    Ok(())
}
