use std::{iter::Copied, pin::Pin};

use futures::Future;
use once_cell::sync::OnceCell;
use radix_trie::{iter::Keys, Trie, TrieCommon};
use twilight_model::application::command::Command;

use crate::{
    commands::{help::*, owner::*, utility::*},
    BotResult,
};

pub use self::command::SlashCommand;

mod command;

macro_rules! slash_trie {
    ($($cmd:ident => $fun:ident,)*) => {
        use twilight_interactions::command::CreateCommand;

        let mut trie = Trie::new();

        $(trie.insert($cmd::NAME, &$fun);)*

        SlashCommands(trie)
    }
}

static SLASH_COMMANDS: OnceCell<SlashCommands> = OnceCell::new();

pub struct SlashCommands(Trie<&'static str, &'static SlashCommand>);

pub type CommandResult = Pin<Box<dyn Future<Output = BotResult<()>> + 'static + Send>>;

type CommandKeys<'t> = Copied<Keys<'t, &'static str, &'static SlashCommand>>;

impl SlashCommands {
    pub fn get() -> &'static Self {
        SLASH_COMMANDS.get_or_init(|| {
            slash_trie! {
                Commands => COMMANDS_SLASH,
                Config => CONFIG_SLASH,
                Help => HELP_SLASH,
                Invite => INVITE_SLASH,
                Owner => OWNER_SLASH,
                Ping => PING_SLASH,
                Prune => PRUNE_SLASH,
                Roll => ROLL_SLASH,
                ServerConfig => SERVERCONFIG_SLASH,
            }
        })
    }

    pub fn command(&self, command: &str) -> Option<&'static SlashCommand> {
        self.0.get(command).copied()
    }

    pub fn collect(&self) -> Vec<Command> {
        self.0.values().map(|c| (c.create)().into()).collect()
    }

    pub fn names(&self) -> CommandKeys<'_> {
        self.0.keys().copied()
    }

    pub fn descendants(&self, prefix: &str) -> Option<CommandKeys<'_>> {
        self.0
            .get_raw_descendant(prefix)
            .map(|sub| sub.keys().copied())
    }
}
