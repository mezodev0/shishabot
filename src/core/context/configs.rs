use std::ptr::null;

use eyre::{Context as _, Result};
use tokio::{fs::OpenOptions, io::AsyncWriteExt};
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

    pub async fn insert_guild_settings(&self, guild_id: Id<GuildMarker>) -> Result<()> {
        let server = Server::default();

        let new_entry = self
            .root_settings
            .servers
            .pin()
            .insert(guild_id, server)
            .is_none();

        if new_entry {
            self.store_guild_settings().await?;
        }

        Ok(())
    }

    pub async fn update_guild_settings<F>(&self, guild_id: Id<GuildMarker>, f: F) -> Result<()>
    where
        F: FnOnce(&mut Server),
    {
        let valid_entry = self
            .root_settings
            .servers
            .pin()
            .compute_if_present(&guild_id, |_, server| {
                let mut server = server.to_owned();
                f(&mut server);

                Some(server)
            })
            .is_some();

        if valid_entry {
            self.store_guild_settings().await?;
        }

        Ok(())
    }

    pub async fn upsert_guild_setings<F>(&self, guild_id: Id<GuildMarker>, f: F) -> Result<()>
    where
        F: FnOnce(&mut Server),
    {
        if !self.root_settings.servers.pin().contains_key(&guild_id) {
            self.insert_guild_settings(guild_id).await;
        }
        self.update_guild_settings(guild_id, f).await;
        Ok(())
    }

    async fn store_guild_settings(&self) -> Result<()> {
        let bytes =
            serde_json::to_vec(&self.root_settings).context("failed to serialize root settings")?;

        let path = BotConfig::get().paths.server_settings();

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)
            .await
            .context("failed to open server settings file")?;

        file.write_all(&bytes)
            .await
            .context("failed writing to server settings file")?;

        Ok(())
    }
}
