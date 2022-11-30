use std::sync::Arc;

use command_macros::SlashCommand;
use eyre::Result;
use twilight_interactions::command::{CommandModel, CreateCommand};

use crate::{
    core::Context,
    util::{interaction::InteractionCommand, InteractionCommandExt},
};

use self::server::{add, remove};

mod server;

#[derive(CommandModel, CreateCommand, SlashCommand)]
#[command(name = "blacklist")]
#[flags(SKIP_DEFER, ONLY_OWNER)]
/// Blacklist a server or user from rendering replays
pub enum Blacklist {
    #[command(name = "server")]
    Server(BlacklistServer),
}

#[derive(CommandModel, CreateCommand)]
#[command(name = "server")]
/// Blacklist a server from rendering replays
pub enum BlacklistServer {
    #[command(name = "add")]
    Add(BlacklistServerAdd),
    #[command(name = "remove")]
    Remove(BlacklistServerRemove),
}

#[derive(CommandModel, CreateCommand)]
#[command(name = "add")]
/// Add a server to the blacklist
pub struct BlacklistServerAdd {
    /// ID of the server you want to blacklist
    guild_id: String,
    /// Reason for the blacklist
    reason: Option<String>,
}

#[derive(CommandModel, CreateCommand)]
#[command(name = "remove")]
/// Remove a server from the blacklist
pub struct BlacklistServerRemove {
    /// ID of the server you want to remove from the blacklist
    _guild_id: String,
}

async fn slash_blacklist(ctx: Arc<Context>, mut command: InteractionCommand) -> Result<()> {
    match Blacklist::from_interaction(command.input_data())? {
        Blacklist::Server(BlacklistServer::Add(args)) => add(ctx, command, args).await,
        Blacklist::Server(BlacklistServer::Remove(args)) => remove(ctx, command, args).await,
    }
}
