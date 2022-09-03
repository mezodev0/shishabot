use std::pin::Pin;

use eyre::Result;
use futures::Future;
use once_cell::sync::OnceCell;
use radix_trie::{Trie, TrieCommon};

use crate::commands::{danser::*, help::*, owner::*, utility::*};

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

pub type CommandResult = Pin<Box<dyn Future<Output = Result<()>> + 'static + Send>>;

impl SlashCommands {
    pub fn get() -> &'static Self {
        SLASH_COMMANDS.get_or_init(|| {
            slash_trie! {
                Help => HELP_SLASH,
                Invite => INVITE_SLASH,
                Owner => OWNER_SLASH,
                Ping => PING_SLASH,
                Queue => QUEUE_SLASH,
                Render => RENDER_SLASH,
                Setup => SETUP_SLASH,
                SkinList => SKINLIST_SLASH,
            }
        })
    }

    pub fn command(&self, command: &str) -> Option<&'static SlashCommand> {
        self.0.get(command).copied()
    }

    pub fn collect<F, O>(&self, f: F) -> Vec<O>
    where
        F: FnMut(&SlashCommand) -> O,
    {
        self.0.values().copied().map(f).collect()
    }
}
