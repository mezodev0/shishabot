use std::{fs, sync::Arc};

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
    let skins_path = BotConfig::get().paths.skins();

    let mut skins = fs::read_dir(&skins_path)
        .context("failed to read skins folder")?
        .map(|res| res.map(|entry| entry.file_name().to_string_lossy().replace('_', " ")))
        .collect::<Result<Vec<_>, _>>()
        .context("failed to read entry of skins folder")?;

    skins.sort_unstable_by_key(|name| name.to_ascii_lowercase());

    SkinListPagination::builder(skins).start(ctx, command).await
}
