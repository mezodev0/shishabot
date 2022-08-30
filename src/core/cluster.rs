use std::{collections::HashMap, sync::Arc};

use eyre::{Context as _, Result};
use twilight_gateway::{cluster::Events, shard::ResumeSession, Cluster, EventTypeFlags, Intents};
use twilight_http::Client;
use twilight_model::gateway::{
    payload::outgoing::update_presence::UpdatePresencePayload,
    presence::{ActivityType, MinimalActivity, Status},
};

use crate::DEFAULT_PREFIX;

pub async fn build_cluster(
    token: &str,
    http: Arc<Client>,
    resume_data: HashMap<u64, ResumeSession>,
) -> Result<(Cluster, Events)> {
    let intents = Intents::GUILDS
        | Intents::GUILD_MEMBERS
        | Intents::GUILD_MESSAGES
        | Intents::DIRECT_MESSAGES
        | Intents::MESSAGE_CONTENT;

    let flags = EventTypeFlags::GATEWAY_INVALIDATE_SESSION
        | EventTypeFlags::GATEWAY_RECONNECT
        | EventTypeFlags::GUILD_CREATE
        | EventTypeFlags::GUILD_DELETE
        | EventTypeFlags::INTERACTION_CREATE
        | EventTypeFlags::MESSAGE_CREATE
        | EventTypeFlags::READY
        | EventTypeFlags::RESUMED
        | EventTypeFlags::SHARD_CONNECTED
        | EventTypeFlags::SHARD_CONNECTING
        | EventTypeFlags::SHARD_DISCONNECTED
        | EventTypeFlags::SHARD_IDENTIFYING
        | EventTypeFlags::SHARD_RECONNECTING
        | EventTypeFlags::SHARD_RESUMING;

    let activity = MinimalActivity {
        kind: ActivityType::Playing,
        name: format!("{DEFAULT_PREFIX}help"),
        url: None,
    };

    let presence =
        UpdatePresencePayload::new([activity.into()], false, None, Status::Online).unwrap();

    let tuple = Cluster::builder(token.to_owned(), intents)
        .event_types(flags)
        .http_client(http)
        .resume_sessions(resume_data)
        .presence(presence)
        .build()
        .await
        .context("failed to build cluster")?;

    Ok(tuple)
}
