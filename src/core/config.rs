use std::{env, path::PathBuf};

use once_cell::sync::OnceCell;
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker, UserMarker},
    Id,
};

use crate::{BotResult, Error};

static CONFIG: OnceCell<BotConfig> = OnceCell::new();

#[derive(Debug)]
pub struct BotConfig {
    pub tokens: Tokens,
    pub paths: Paths,
    pub owners: Vec<Id<UserMarker>>,
    pub dev_guild: Id<GuildMarker>,
}

#[derive(Debug)]
pub struct Paths {
    pub backgrounds: PathBuf,
    pub cards: PathBuf,
    pub maps: PathBuf,
    pub website: PathBuf,
}

#[derive(Debug)]
pub struct Tokens {
    pub discord: String,
    pub osu_client_id: u64,
    pub osu_client_secret: String,
}

impl BotConfig {
    pub fn get() -> &'static Self {
        CONFIG
            .get()
            .expect("`BotConfig::init` must be called first")
    }

    pub fn init() -> BotResult<()> {
        let config = BotConfig {
            tokens: Tokens {
                discord: env_var("DISCORD_TOKEN")?,
                osu_client_id: env_var("OSU_CLIENT_ID")?,
                osu_client_secret: env_var("OSU_CLIENT_SECRET")?,
            },
            paths: Paths {
                backgrounds: env_var("BG_PATH")?,
                cards: env_var("CARDS_REPO_PATH")?,
                maps: env_var("MAP_PATH")?,
                website: env_var("WEBSITE_PATH")?,
            },
            owners: env_var("OWNERS_USER_ID")?,
            dev_guild: env_var("DEV_GUILD_ID")?,
        };

        if CONFIG.set(config).is_err() {
            warn!("CONFIG was already set");
        }

        Ok(())
    }
}

trait EnvKind: Sized {
    const EXPECTED: &'static str;

    fn from_str(s: &str) -> Option<Self>;
}

macro_rules! env_kind {
    ($($ty:ty: $arg:ident => $impl:block,)*) => {
        $(
            impl EnvKind for $ty {
                const EXPECTED: &'static str = stringify!($ty);

                fn from_str($arg: &str) -> Option<Self> {
                    $impl
                }
            }
        )*
    };
}

env_kind! {
    u16: s => { s.parse().ok() },
    u64: s => { s.parse().ok() },
    PathBuf: s => { s.parse().ok() },
    String: s => { Some(s.to_owned()) },
    Id<UserMarker>: s => { s.parse().ok().map(Id::new) },
    Id<GuildMarker>: s => { s.parse().ok().map(Id::new) },
    Id<ChannelMarker>: s => { s.parse().ok().map(Id::new) },
    Vec<Id<UserMarker>>: s => {
        if !(s.starts_with('[') && s.ends_with(']')) {
            return None
        }

        s[1..s.len() - 1]
            .split(',')
            .map(str::trim)
            .map(str::parse)
            .collect::<Result<Vec<_>, _>>()
            .ok()
    },
}

fn env_var<T: EnvKind>(name: &'static str) -> BotResult<T> {
    let value = env::var(name).map_err(|_| Error::MissingEnvVariable(name))?;

    T::from_str(&value).ok_or(Error::ParsingEnvVariable {
        name,
        value,
        expected: T::EXPECTED,
    })
}
