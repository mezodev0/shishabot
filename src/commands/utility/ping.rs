use std::{sync::Arc, time::Instant};

use command_macros::{command, SlashCommand};
use eyre::Result;
use twilight_interactions::command::CreateCommand;

use crate::{
    core::Context,
    util::{
        builder::MessageBuilder, interaction::InteractionCommand, InteractionCommandExt, MessageExt,
    },
};

#[derive(CreateCommand, SlashCommand)]
#[command(
    name = "ping",
    help = "Most basic command, generally used to check if the bot is online.\n\
    The displayed latency is the time it takes for the bot \
    to receive a response from discord after sending a message."
)]
#[flags(SKIP_DEFER)]
/// Check if the bot is online
pub struct Ping;

async fn slash_ping(ctx: Arc<Context>, mut command: InteractionCommand) -> Result<()> {
    let builder = MessageBuilder::new().content("Pong");
    let start = Instant::now();
    let response_raw = command.callback(&ctx, builder, false).await?;
    let elapsed = (Instant::now() - start).as_millis();

    let response = ctx
        .interaction()
        .response(&command.token)
        .exec()
        .await?
        .model()
        .await?;

    let content = format!(":ping_pong: Pong! ({elapsed}ms)");
    let builder = MessageBuilder::new().content(content);
    response.update(&ctx, &builder).await?;

    Ok(())
}
