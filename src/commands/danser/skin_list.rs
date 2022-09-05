use std::sync::Arc;

use command_macros::SlashCommand;
use eyre::Result;
use twilight_interactions::command::{CommandModel, CreateCommand};

use crate::{core::Context, pagination::SkinListPagination, util::interaction::InteractionCommand};

#[derive(CreateCommand, CommandModel, SlashCommand)]
#[command(name = "skinlist")]
#[flags(SKIP_DEFER)]
/// Displays all available skins
pub struct SkinList;

async fn slash_skinlist(ctx: Arc<Context>, command: InteractionCommand) -> Result<()> {
    let skins = ctx
        .skin_list()
        .get()?
        .iter()
        .map(|skin| skin.to_string_lossy().replace('_', " "))
        .collect();

    SkinListPagination::builder(skins).start(ctx, command).await
}
