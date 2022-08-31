use std::pin::Pin;

use eyre::Result;
use futures::Future;
use once_cell::sync::OnceCell;
use radix_trie::{Trie, TrieCommon};

use crate::commands::{danser::*, help::HELP_PREFIX, utility::*};

pub use self::{args::Args, command::PrefixCommand, stream::Stream};

mod args;
mod command;
mod stream;

macro_rules! prefix_trie {
    ($($cmd:ident,)*) => {
        let mut trie = Trie::new();

        $(
            for &name in $cmd.names {
                if trie.insert(name, &$cmd).is_some() {
                    panic!("duplicate prefix command `{name}`");
                }
            }
        )*

        PrefixCommands(trie)
    }
}

static PREFIX_COMMANDS: OnceCell<PrefixCommands> = OnceCell::new();

pub type CommandResult<'fut> = Pin<Box<dyn Future<Output = Result<()>> + 'fut + Send>>;

type PrefixTrie = Trie<&'static str, &'static PrefixCommand>;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrefixCommandGroup {
    AllModes,
    Osu,
    Taiko,
    Catch,
    Mania,
    Tracking,
    Twitch,
    Games,
    Danser,
    Utility,
}

impl PrefixCommandGroup {
    pub fn emote(self) -> &'static str {
        match self {
            Self::Danser => ":question:",
            Self::Utility => ":tools:",
            _ => panic!("no emote for {self:?}"),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::AllModes => "all osu! modes",
            Self::Osu => "osu!standard",
            Self::Taiko => "osu!taiko",
            Self::Catch => "osu!catch",
            Self::Mania => "osu!mania",
            Self::Tracking => "osu!tracking",
            Self::Twitch => "twitch",
            Self::Games => "games",
            Self::Danser => "danser",
            Self::Utility => "utility",
        }
    }
}

pub struct PrefixCommands(PrefixTrie);

impl PrefixCommands {
    pub fn get() -> &'static Self {
        PREFIX_COMMANDS.get_or_init(|| {
            prefix_trie! {
                COMMANDS_PREFIX,
                INVITE_PREFIX,
                HELP_PREFIX,
                PING_PREFIX,
                PREFIX_PREFIX,
                PRUNE_PREFIX,
                QUEUE_PREFIX,
                ROLL_PREFIX,
            }
        })
    }

    pub fn command(&self, command: &str) -> Option<&'static PrefixCommand> {
        self.0.get(command).copied()
    }

    pub fn iter(&self) -> impl Iterator<Item = &'static PrefixCommand> + '_ {
        self.0.values().copied()
    }
}
