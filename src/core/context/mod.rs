use std::sync::Arc;

use flexmap::tokio::TokioMutexMap;
use flurry::HashMap as FlurryMap;
use rosu_v2::Osu;
use twilight_gateway::{cluster::Events, Cluster};
use twilight_http::{client::InteractionClient, Client};
use twilight_model::{
    channel::message::allowed_mentions::AllowedMentionsBuilder,
    id::{
        marker::{ApplicationMarker, GuildMarker, MessageMarker},
        Id,
    },
};
use twilight_standby::Standby;

use crate::{
    core::BotConfig, custom_client::CustomClient, pagination::Pagination,
    util::hasher::SimpleBuildHasher, BotResult,
};

use super::{buckets::Buckets, cluster::build_cluster, Cache};

mod configs;

pub struct Context {
    pub buckets: Buckets,
    pub cache: Cache,
    pub cluster: Cluster,
    pub http: Arc<Client>,
    pub paginations: Arc<TokioMutexMap<Id<MessageMarker>, Pagination, SimpleBuildHasher>>,
    pub standby: Standby,
    data: ContextData,
    clients: Clients,
}

impl Context {
    pub fn interaction(&self) -> InteractionClient<'_> {
        self.http.interaction(self.data.application_id)
    }

    pub fn osu(&self) -> &Osu {
        &self.clients.osu
    }

    /// Returns the custom client
    pub fn client(&self) -> &CustomClient {
        &self.clients.custom
    }

    pub async fn new() -> BotResult<(Self, Events)> {
        let config = BotConfig::get();
        let discord_token = &config.tokens.discord;

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

        // Connect to osu! API
        let osu_client_id = config.tokens.osu_client_id;
        let osu_client_secret = &config.tokens.osu_client_secret;
        let osu = Osu::new(osu_client_id, osu_client_secret).await?;

        // Log custom client into osu!
        let custom = CustomClient::new().await?;

        let data = ContextData::new(application_id).await?;
        let (cache, resume_data) = Cache::new().await;

        let clients = Clients::new(osu, custom);
        let (cluster, events) =
            build_cluster(discord_token, Arc::clone(&http), resume_data).await?;

        let ctx = Self {
            cache,
            http,
            clients,
            cluster,
            data,
            standby: Standby::new(),
            buckets: Buckets::new(),
            paginations: Arc::new(TokioMutexMap::with_shard_amount_and_hasher(
                16,
                SimpleBuildHasher,
            )),
        };

        Ok((ctx, events))
    }
}

struct Clients {
    custom: CustomClient,
    osu: Osu,
}

impl Clients {
    fn new(osu: Osu, custom: CustomClient) -> Self {
        Self { osu, custom }
    }
}

struct ContextData {
    application_id: Id<ApplicationMarker>,
    guilds: FlurryMap<Id<GuildMarker>, GuildConfig, SimpleBuildHasher>, // read-heavy
}

// TODO
pub type GuildConfig = ();

impl ContextData {
    async fn new(application_id: Id<ApplicationMarker>) -> BotResult<Self> {
        Ok(Self {
            application_id,
            guilds: FlurryMap::default(),
        })
    }
}
