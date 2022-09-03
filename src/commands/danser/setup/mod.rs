use std::{process::Output, sync::Arc};

use command_macros::SlashCommand;
use eyre::Result;
use twilight_interactions::command::{CommandModel, CommandOption, CreateCommand, CreateOption};
use twilight_model::id::{marker::ChannelMarker, Id};

use crate::{
    commands::server_administrator,
    util::{interaction::InteractionCommand, InteractionCommandExt},
    Context,
};

use self::{input::*, output::*, view::*};

mod input;
mod output;
mod view;

#[derive(CommandModel, CreateCommand, SlashCommand)]
#[command(name = "setup", dm_permission = false)]
#[flags(SKIP_DEFER)]
/// Channel setup for the bot
pub enum Setup {
    #[command(name = "view")]
    View(SetupView),
    #[command(name = "input")]
    Input(SetupInput),
    #[command(name = "output")]
    Output(SetupOutput),
}

#[derive(CommandModel, CreateCommand)]
#[command(name = "view")]
/// Shows the current configuration of the channels
pub struct SetupView;

#[derive(CommandModel, CreateCommand)]
#[command(name = "input", default_permissions = "server_administrator")]
/// Configure the the channels in which replays can be rendered
pub struct SetupInput {
    /// Add or remove a channel
    action: InputAction,
    /// The channel you want to add/remove
    channel: Id<ChannelMarker>,
}

#[derive(CommandOption, CreateOption)]
pub enum InputAction {
    #[option(name = "add", value = "add")]
    Add,
    #[option(name = "remove", value = "remove")]
    Remove,
}

#[derive(CommandModel, CreateCommand)]
#[command(name = "output", default_permissions = "server_administrator")]
/// Configure the the channel in which the replay will be sent
pub struct SetupOutput {
    /// The channel you want as the output channel
    channel: Id<ChannelMarker>,
}

async fn slash_setup(ctx: Arc<Context>, mut command: InteractionCommand) -> Result<()> {
    match Setup::from_interaction(command.input_data())? {
        Setup::View(_) => view(ctx, command).await,
        Setup::Input(args) => input(ctx, command, args).await,
        Setup::Output(args) => output(ctx, command, args).await,
    }
}
