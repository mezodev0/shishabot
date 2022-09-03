use std::sync::Arc;

use command_macros::{command, SlashCommand};
use eyre::{Context as _, Result};
use tokio::fs::{self, ReadDir};
use twilight_interactions::command::{CommandModel, CreateCommand};

use crate::{
    core::{BotConfig, Context},
    pagination::SkinListPagination,
    util::{constants::GENERAL_ISSUE, interaction::InteractionCommand, InteractionCommandExt},
};

#[derive(CreateCommand, CommandModel, SlashCommand)]
#[command(name = "skinlist")]
#[flags(SKIP_DEFER)]
/// Displays all skins available
pub struct SkinList;

async fn slash_skinlist(ctx: Arc<Context>, mut command: InteractionCommand) -> Result<()> {
    let mut skins_path = BotConfig::get().paths.skins();

    let mut dir = match fs::read_dir(&skins_path).await {
        Ok(dir) => dir,
        Err(err) => {
            let _ = command.error_callback(&ctx, GENERAL_ISSUE, false).await;

            return Err(err).with_context(|| format!("failed to read {skins_path:?} directory"));
        }
    };

    let mut skins = Vec::new();

    loop {
        match dir.next_entry().await {
            Ok(Some(entry)) => {
                let file_name = entry.file_name();
                let name = file_name.to_string_lossy().replace('_', " ");
                skins.push(name);
            }
            Ok(None) => break,
            Err(err) => {
                let _ = command.error_callback(&ctx, GENERAL_ISSUE, false).await;

                return Err(err)
                    .with_context(|| format!("failed to get next entry in {skins_path:?}"));
            }
        }
    }

    SkinListPagination::builder(skins).start(ctx, command).await
}
