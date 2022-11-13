use std::pin::Pin;

use eyre::Result;
use futures::Future;
use once_cell::sync::OnceCell;
use radix_trie::{Trie, TrieCommon};
use twilight_interactions::command::CreateCommand;

use crate::commands::{danser::*, help::*, owner::*, utility::*};

pub use self::command::{Command, MessageCommand, SlashCommand};

mod command;

macro_rules! slash_trie {
    (
        slash {
            $($slash_cmd:ident => $chat_fun:ident,)*
        },
        msg {
            $($msg_cmd:ident,)*
        }
    ) => {
        let mut trie = Trie::new();

        $(trie.insert($slash_cmd::NAME, Command::Slash(&$chat_fun));)*
        $(trie.insert($msg_cmd.name, Command::Message(&$msg_cmd));)*

        Commands(trie)
    }
}

static COMMANDS: OnceCell<Commands> = OnceCell::new();

pub struct Commands(Trie<&'static str, Command>);

pub type CommandResult = Pin<Box<dyn Future<Output = Result<()>> + 'static + Send>>;

impl Commands {
    pub fn get() -> &'static Self {
        COMMANDS.get_or_init(|| {
            slash_trie! {
                slash {
                    Help => HELP_SLASH,
                    Invite => INVITE_SLASH,
                    Owner => OWNER_SLASH,
                    Ping => PING_SLASH,
                    Queue => QUEUE_SLASH,
                    Render => RENDER_SLASH,
                    Setup => SETUP_SLASH,
                    Skin => SKIN_SLASH,
                    Settings => SETTINGS_SLASH,
                    Setup => SETUP_SLASH,
                },
                msg {
                    RENDER_FROM_MSG,
                }
            }
        })
    }

    pub fn command(&self, command: &str) -> Option<Command> {
        self.0.get(command).copied()
    }

    pub fn collect<F, O>(&self, f: F) -> Vec<O>
    where
        F: FnMut(&Command) -> O,
    {
        self.0.values().map(f).collect()
    }

    pub fn filter_collect<F, O>(&self, f: F) -> Vec<O>
    where
        F: FnMut(Command) -> Option<O>,
    {
        self.0.values().copied().filter_map(f).collect()
    }
}
