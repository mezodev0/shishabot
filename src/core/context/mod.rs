use std::sync::{Arc, Mutex, MutexGuard};

use eyre::{Result, WrapErr};
use flexmap::tokio::TokioMutexMap;
use rosu_v2::Osu;
use tokio::{fs, io::AsyncWriteExt};
use twilight_gateway::{cluster::Events, Cluster};
use twilight_http::{client::InteractionClient, Client};
use twilight_model::{
    channel::message::allowed_mentions::AllowedMentionsBuilder,
    id::{
        marker::{ApplicationMarker, MessageMarker},
        Id,
    },
};
use twilight_standby::Standby;

use crate::{
    core::BotConfig, custom_client::CustomClient, database::Database, pagination::Pagination,
    util::hasher::IntBuildHasher,
};

use super::{cluster::build_cluster, settings::RootSettings, stats::BotStats, Cache, ReplayQueue};

use self::skin_list::SkinList;

mod configs;
mod skin_list;

pub struct Context {
    pub cache: Cache,
    pub cluster: Cluster,
    pub http: Arc<Client>,
    pub paginations: Arc<TokioMutexMap<Id<MessageMarker>, Pagination, IntBuildHasher>>,
    pub standby: Standby,
    pub stats: Arc<BotStats>,
    pub replay_queue: ReplayQueue,
    root_settings: RootSettings,
    skin_list: Arc<Mutex<SkinList>>,
    application_id: Id<ApplicationMarker>,
    clients: Clients,
}

impl Context {
    pub fn interaction(&self) -> InteractionClient<'_> {
        self.http.interaction(self.application_id)
    }

    pub fn osu(&self) -> &Osu {
        &self.clients.osu
    }

    pub fn psql(&self) -> &Database {
        &self.clients.psql
    }

    pub fn client(&self) -> &CustomClient {
        &self.clients.custom
    }

    /// Returns a mutex guard, don't hold it for too long!
    pub fn skin_list(&self) -> MutexGuard<'_, SkinList> {
        self.skin_list.lock().unwrap()
    }

    pub async fn new() -> Result<(Self, Events)> {
        let config = BotConfig::get();

        create_missing_folders_and_files(config).await?;

        let discord_token = &config.tokens.discord;

        let bytes = fs::read(config.paths.server_settings())
            .await
            .context("failed to read server settings file")?;

        let root_settings =
            serde_json::from_slice(&bytes).context("failed to deserialize server settings file")?;

        let mentions = AllowedMentionsBuilder::new()
            .replied_user()
            .roles()
            .users()
            .build();

        // Connect to the discord http client
        let http = Client::builder()
            .token(discord_token.to_owned())
            .remember_invalid_token(false)
            .default_allowed_mentions(mentions)
            .build();

        let http = Arc::new(http);

        let current_user = http.current_user().exec().await?.model().await?;
        let application_id = current_user.id.cast();

        info!(
            "Connecting to Discord as {}#{}...",
            current_user.name, current_user.discriminator
        );

        // Connect to psql database
        let psql =
            Database::new(&config.database_url).wrap_err("failed to create database client")?;

        // Connect to osu! API
        let osu_client_id = config.tokens.osu_client_id;
        let osu_client_secret = &config.tokens.osu_client_secret;
        let osu = Osu::new(osu_client_id, osu_client_secret).await?;

        let custom = CustomClient::new();

        let (cache, resume_data) = Cache::new().await;
        let stats = Arc::new(BotStats::new());

        let clients = Clients::new(osu, custom, psql);
        let (cluster, events) =
            build_cluster(discord_token, Arc::clone(&http), resume_data).await?;

        let paginations = TokioMutexMap::with_shard_amount_and_hasher(16, IntBuildHasher);

        let ctx = Self {
            cache,
            http,
            clients,
            cluster,
            application_id,
            root_settings,
            paginations: Arc::new(paginations),
            standby: Standby::new(),
            stats,
            replay_queue: ReplayQueue::new(),
            skin_list: Arc::new(Mutex::default()),
        };

        Ok((ctx, events))
    }
}

struct Clients {
    custom: CustomClient,
    osu: Osu,
    psql: Database,
}

impl Clients {
    fn new(osu: Osu, custom: CustomClient, psql: Database) -> Self {
        Self { osu, custom, psql }
    }
}

async fn create_missing_folders_and_files(config: &BotConfig) -> Result<()> {
    fs::create_dir_all(config.paths.downloads())
        .await
        .context("failed to create Downloads folder")?;

    fs::create_dir_all(config.paths.replays())
        .await
        .context("failed to create Replays folder")?;

    fs::create_dir_all(config.paths.skins())
        .await
        .context("failed to create Skins folder")?;

    fs::create_dir_all(config.paths.songs())
        .await
        .context("failed to create Songs folder")?;

    let danser_entry = config
        .paths
        .danser()
        .read_dir()
        .ok()
        .and_then(|mut dir| dir.next());

    ensure!(
        danser_entry.is_some(),
        "danser not found! please download from https://github.com/Wieku/danser-go/releases/"
    );

    let server_settings = config.paths.server_settings();

    if !server_settings.exists() {
        let mut file = fs::File::create(server_settings)
            .await
            .context("failed to create server settings file")?;

        file.write_all(b"{\"Servers\":[]}")
            .await
            .context("failed writing to server settings file")?;
    }

    Ok(())
}
