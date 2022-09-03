use std::{fs::OpenOptions, io::Write};

use eyre::{Context as _, Result};
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker},
    Id,
};

use crate::{
    core::{settings::Server, BotConfig},
    Context,
};

impl Context {
    pub fn guild_settings<F, O>(&self, guild_id: Id<GuildMarker>, f: F) -> Option<O>
    where
        F: FnOnce(&Server) -> O,
    {
        self.root_settings.servers.pin().get(&guild_id).map(f)
    }

    pub fn upsert_guild_settings<F, O>(&self, guild_id: Id<GuildMarker>, f: F) -> Result<O>
    where
        F: FnOnce(&mut Server) -> O,
    {
        let (output, entry_was_present) = {
            let guard = self.root_settings.servers.guard();

            let mut server = self
                .root_settings
                .servers
                .get(&guild_id, &guard)
                .cloned()
                .unwrap_or_default();

            let output = f(&mut server);

            let entry_was_present = self
                .root_settings
                .servers
                .insert(guild_id, server, &guard)
                .is_some();

            (output, entry_was_present)
        };

        if entry_was_present {
            self.store_guild_settings()
                .context("failed to upsert server settings")?;
        }

        Ok(output)
    }

    fn store_guild_settings(&self) -> Result<()> {
        let path = BotConfig::get().paths.server_settings();

        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)
            .context("failed to open server settings file")?;

        serde_json::to_writer(file, &self.root_settings)
            .context("failed to serialize root settings")?;

        Ok(())
    }
}
