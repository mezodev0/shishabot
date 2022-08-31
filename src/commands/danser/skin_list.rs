use std::sync::Arc;

use command_macros::{command, SlashCommand};
use eyre::{Context as _, Result};
use tokio::fs::{self, ReadDir};
use twilight_interactions::command::{CommandModel, CreateCommand};

use crate::{
    core::{commands::CommandOrigin, BotConfig, Context},
    pagination::SkinListPagination,
    util::{constants::GENERAL_ISSUE, interaction::InteractionCommand},
};

#[derive(CreateCommand, CommandModel, SlashCommand)]
#[command(name = "skinlist")]
#[flags(SKIP_DEFER)]
/// Displays all skins available
pub struct SkinList;

async fn slash_skinlist(ctx: Arc<Context>, mut command: InteractionCommand) -> Result<()> {
    skin_list(ctx, (&mut command).into()).await
}

#[command]
#[desc("Displays all skins available")]
#[alias("sl")]
#[flags(SKIP_DEFER)]
#[group(Danser)]
async fn prefix_skinlist(ctx: Arc<Context>, msg: &Message) -> Result<()> {
    skin_list(ctx, msg.into()).await
}

async fn skin_list(ctx: Arc<Context>, orig: CommandOrigin<'_>) -> Result<()> {
    let mut skins_path = BotConfig::get().paths.folders.clone();
    skins_path.push("Skins");

    let mut dir = match fs::read_dir(&skins_path).await {
        Ok(dir) => dir,
        Err(err) => {
            let _ = orig.error(&ctx, GENERAL_ISSUE).await;

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
                let _ = orig.error(&ctx, GENERAL_ISSUE).await;

                return Err(err)
                    .with_context(|| format!("failed to get next entry in {skins_path:?}"));
            }
        }
    }

    SkinListPagination::builder(skins).start(ctx, orig).await
}
