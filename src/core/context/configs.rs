use eyre::{Context as _, Result};
use tokio::{fs::OpenOptions, io::AsyncWriteExt};
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker},
    Id,
};

use crate::{
    core::{
        commands::prefix::Stream,
        settings::{Prefix, Prefixes, Server},
        BotConfig,
    },
    Context, DEFAULT_PREFIX,
};

impl Context {
    pub fn guild_settings<F, O>(&self, guild_id: Id<GuildMarker>, f: F) -> Option<O>
    where
        F: FnOnce(&Server) -> O,
    {
        self.root_settings.servers.pin().get(&guild_id).map(f)
    }

    pub fn guild_prefixes(&self, guild_id: Id<GuildMarker>) -> Prefixes {
        self.guild_settings(guild_id, |server| server.prefixes.clone())
            .unwrap_or_else(|| smallvec::smallvec![DEFAULT_PREFIX.into()])
    }

    pub fn guild_prefixes_find(
        &self,
        guild_id: Id<GuildMarker>,
        stream: &Stream<'_>,
    ) -> Option<Prefix> {
        let f = |server: &Server| {
            server
                .prefixes
                .iter()
                .find(|p| stream.starts_with(p))
                .cloned()
        };

        self.guild_settings(guild_id, f).flatten()
    }

    pub async fn guild_first_prefix(&self, guild_id: Option<Id<GuildMarker>>) -> Prefix {
        guild_id
            .and_then(|id| self.guild_settings(id, |server| server.prefixes.get(0).cloned()))
            .flatten()
            .unwrap_or_else(|| DEFAULT_PREFIX.into())
    }

    pub async fn insert_guild_settings(
        &self,
        guild_id: Id<GuildMarker>,
        input_channel: Id<ChannelMarker>,
        output_channel: Id<ChannelMarker>,
    ) -> Result<()> {
        let server = Server::new(input_channel, output_channel);

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

    async fn store_guild_settings(&self) -> Result<()> {
        let bytes =
            serde_json::to_vec(&self.root_settings).context("failed to serialize root settings")?;

        let path = &BotConfig::get().paths.server_settings;

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
