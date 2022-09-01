use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    sync::Arc,
};

use eyre::{Context as _, Result};
use futures::StreamExt;
use twilight_gateway::{cluster::Events, Event};
use twilight_model::gateway::{
    payload::outgoing::UpdatePresence,
    presence::{ActivityType, MinimalActivity, Status},
};

use crate::{util::Authored, DEFAULT_PREFIX};

use self::{interaction::handle_interaction, message::handle_message};

use super::{buckets::BucketName, Context};

mod interaction;
mod message;

#[derive(Debug)]
enum ProcessResult {
    Success,
    NoDM,
    NoSendPermission,
    Ratelimited(BucketName),
    NoOwner,
    NoAuthority,
}

fn log_command(ctx: &Context, cmd: &dyn Authored, name: &str) {
    let username = cmd
        .user()
        .map(|u| u.name.as_str())
        .unwrap_or("<unknown user>");

    let location = CommandLocation { ctx, cmd };
    info!("[{location}] {username} invoked `{name}`");
}

struct CommandLocation<'a> {
    ctx: &'a Context,
    cmd: &'a dyn Authored,
}

impl Display for CommandLocation<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let guild = match self.cmd.guild_id() {
            Some(id) => id,
            None => return f.write_str("Private"),
        };

        match self.ctx.cache.guild(guild, |g| write!(f, "{}:", g.name())) {
            Ok(Ok(_)) => {
                let channel_res = self.ctx.cache.channel(self.cmd.channel_id(), |c| {
                    f.write_str(c.name.as_deref().unwrap_or("<uncached channel>"))
                });

                match channel_res {
                    Ok(Ok(_)) => Ok(()),
                    Ok(err) => err,
                    Err(_) => f.write_str("<uncached channel>"),
                }
            }
            Ok(err) => err,
            Err(_) => f.write_str("<uncached guild>"),
        }
    }
}

pub async fn event_loop(ctx: Arc<Context>, mut events: Events) {
    while let Some((shard_id, event)) = events.next().await {
        ctx.cache.update(&event);
        ctx.standby.process(&event);
        let ctx = Arc::clone(&ctx);

        tokio::spawn(async move {
            let handle_fut = handle_event(ctx, event, shard_id);

            if let Err(err) = handle_fut.await.context("error while handling event") {
                error!("{err:?}");
            }
        });
    }
}

async fn handle_event(ctx: Arc<Context>, event: Event, shard_id: u64) -> Result<()> {
    match event {
        Event::GatewayInvalidateSession(true) => {
            warn!("Gateway invalidated session for shard {shard_id}, but its reconnectable")
        }
        Event::GatewayInvalidateSession(false) => {
            warn!("Gateway invalidated session for shard {shard_id}")
        }
        Event::GatewayReconnect => {
            info!("Gateway requested shard {shard_id} to reconnect")
        }
        Event::GuildCreate(_) | Event::GuildDelete(_) => {
            let count = ctx.cache.stats().guilds();

            let activity = MinimalActivity {
                kind: ActivityType::Watching,
                name: format!("in {count} servers | {DEFAULT_PREFIX}help"),
                url: None,
            };

            let req = UpdatePresence::new(vec![activity.into()], false, None, Status::Online)?;

            ctx.cluster
                .command(shard_id, &req)
                .await
                .context("failed to update activity")?;
        }
        Event::InteractionCreate(e) => handle_interaction(ctx, e.0).await,
        Event::MessageCreate(msg) => handle_message(ctx, msg.0).await,
        Event::Ready(_) => info!("Shard {shard_id} is ready"),
        Event::Resumed => info!("Shard {shard_id} is resumed"),
        Event::ShardConnected(_) => info!("Shard {shard_id} is connected"),
        Event::ShardConnecting(_) => info!("Shard {shard_id} is connecting..."),
        Event::ShardDisconnected(_) => info!("Shard {shard_id} is disconnected"),
        Event::ShardIdentifying(_) => info!("Shard {shard_id} is identifying..."),
        Event::ShardReconnecting(_) => info!("Shard {shard_id} is reconnecting..."),
        Event::ShardResuming(_) => info!("Shard {shard_id} is resuming..."),
        _ => {}
    }

    Ok(())
}
