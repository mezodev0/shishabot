use std::sync::Arc;

use command_macros::SlashCommand;
use eyre::Result;

use twilight_interactions::command::{CommandModel, CreateCommand};

use crate::{core::Context, util::interaction::InteractionCommand};

#[derive(CreateCommand, CommandModel, SlashCommand)]
#[command(name = "test")]
#[flags(SKIP_DEFER)]
/// Displays the current replay queue
pub struct Test;

async fn slash_test(ctx: Arc<Context>, command: InteractionCommand) -> Result<()> {
    ctx.psql()
    Ok(())
}
