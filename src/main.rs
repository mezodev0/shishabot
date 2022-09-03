// TODO: uncomment
// #![deny(clippy::all, nonstandard_style, rust_2018_idioms, unused, warnings)]
// #![allow(unused)]

#[macro_use]
extern crate eyre;

#[macro_use]
extern crate tracing;

mod commands;
mod core;
mod custom_client;
mod pagination;
mod util;

use std::sync::Arc;

use eyre::{Context as _, Result};
use tokio::{runtime::Builder as RuntimeBuilder, signal};

use crate::core::{
    commands::slash::SlashCommands, event_loop, logging, BotConfig, Context, ReplayQueue,
};

fn main() {
    let runtime = RuntimeBuilder::new_multi_thread()
        .enable_all()
        .thread_stack_size(4 * 1024 * 1024)
        .build()
        .expect("Could not build runtime");

    if let Err(err) = runtime.block_on(async_main()) {
        error!("critical error in main: {err:?}");
    }
}

async fn async_main() -> Result<()> {
    let _ = dotenv::dotenv().expect("failed to parse .env file");
    let _log_worker_guard = logging::initialize();

    // Load config file
    BotConfig::init().context("failed to initialize config")?;

    let (ctx, events) = Context::new().await.context("failed to create ctx")?;

    let ctx = Arc::new(ctx);

    // Initialize commands
    let slash_commands = SlashCommands::get().collect();
    info!("Setting {} slash commands...", slash_commands.len());

    // info!("Defining: {slash_commands:#?}");

    if cfg!(debug_assertions) {
        ctx.interaction()
            .set_global_commands(&[])
            .exec()
            .await
            .context("failed to set empty global commands")?;

        let _received = ctx
            .interaction()
            .set_guild_commands(BotConfig::get().dev_guild, &slash_commands)
            .exec()
            .await
            .context("failed to set guild commands")?;

        // let commands = _received.models().await?;
        // info!("Received: {commands:#?}");
    } else {
        ctx.interaction()
            .set_global_commands(&slash_commands)
            .exec()
            .await
            .context("failed to set global commands")?;
    }

    let event_ctx = Arc::clone(&ctx);
    ctx.cluster.up().await;

    // Process the replay queue in the background
    ReplayQueue::process(Arc::clone(&ctx));

    tokio::select! {
        _ = event_loop(event_ctx, events) => error!("Event loop ended"),
        res = signal::ctrl_c() => if let Err(err) = res.context("error while awaiting ctrl+c") {
            error!("{err:?}");
        } else {
            info!("Received Ctrl+C");
        },
    }

    ctx.cluster.down();

    info!("Shutting down");

    Ok(())
}
