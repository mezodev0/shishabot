use std::sync::Arc;

use command_macros::SlashCommand;
use twilight_interactions::command::{CommandModel, CreateCommand};

use crate::{
    util::{interaction::InteractionCommand, InteractionCommandExt},
    BotResult, Context,
};

use self::cache::*;

mod cache;

#[derive(CommandModel, CreateCommand, SlashCommand)]
#[command(name = "owner")]
#[flags(ONLY_OWNER, SKIP_DEFER)]
/// You won't be able to use this :^)
pub enum Owner {
    #[command(name = "cache")]
    Cache(OwnerCache),
}

#[derive(CommandModel, CreateCommand)]
#[command(name = "cache")]
/// Display stats about the internal cache
pub struct OwnerCache;

// * EXAMPLE:
// #[derive(CommandModel, CreateCommand)]
// #[command(name = "interval")]
// /// Adjust the tracking interval
// pub struct OwnerTrackingInterval {
//     /// Specify the interval in seconds, defaults to 9000
//     number: Option<i64>,
// }

async fn slash_owner(ctx: Arc<Context>, mut command: InteractionCommand) -> BotResult<()> {
    match Owner::from_interaction(command.input_data())? {
        Owner::Cache(_) => cache(ctx, command).await,
    }
}
