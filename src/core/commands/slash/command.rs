use std::sync::Arc;

use twilight_interactions::command::ApplicationCommandData;
use twilight_model::application::command::Command as TwilightCommand;

use crate::{
    core::{commands::flags::CommandFlags, Context},
    util::interaction::InteractionCommand,
};

use super::CommandResult;

#[derive(Copy, Clone)]
pub enum Command {
    Slash(&'static SlashCommand),
    Message(&'static MessageCommand),
}

impl Command {
    pub fn create(&self) -> TwilightCommand {
        match self {
            Command::Slash(cmd) => (cmd.create)().into(),
            Command::Message(cmd) => (cmd.create)(),
        }
    }
}

pub struct SlashCommand {
    pub create: fn() -> ApplicationCommandData,
    pub exec: fn(Arc<Context>, InteractionCommand) -> CommandResult,
    pub flags: CommandFlags,
}

pub struct MessageCommand {
    pub create: fn() -> TwilightCommand,
    pub exec: fn(Arc<Context>, InteractionCommand) -> CommandResult,
    pub name: &'static str,
}
