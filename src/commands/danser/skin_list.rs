use std::{ffi::OsString, fs, sync::Arc};

use command_macros::SlashCommand;
use eyre::{Context as _, Result};
use twilight_interactions::command::{CommandModel, CreateCommand};

use crate::{
    core::{BotConfig, Context},
    pagination::SkinListPagination,
    util::{
        constants::GENERAL_ISSUE, interaction::InteractionCommand, CowUtils, InteractionCommandExt,
    },
};

#[derive(CreateCommand, CommandModel, SlashCommand)]
#[command(name = "skinlist")]
#[flags(SKIP_DEFER)]
/// Displays all skins available
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
