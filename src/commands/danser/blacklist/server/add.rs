use std::sync::Arc;

use crate::{
    commands::danser::BlacklistServerAdd,
    core::Context,
    util::{builder::MessageBuilder, interaction::InteractionCommand, InteractionCommandExt},
};
use eyre::Result;

pub async fn add(
    ctx: Arc<Context>,
    command: InteractionCommand,
    args: BlacklistServerAdd,
) -> Result<()> {
    let BlacklistServerAdd { guild_id, reason } = args;

    let guild_id_parsed = match guild_id.parse::<u64>() {
        Ok(guild_id) => guild_id,
        Err(_) => {
            command
                .error_callback(&ctx, "Guild ID is invalid!", true)
                .await?;
            return Ok(());
        }
    };

    ctx.psql().blacklist_server(guild_id_parsed, reason).await?;
    let builder =
        MessageBuilder::new().embed(format!("Server `{guild_id}` successfully blacklisted"));
    command.callback(&ctx, builder, false).await?;
    Ok(())
}
