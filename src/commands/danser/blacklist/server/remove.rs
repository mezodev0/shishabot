use std::sync::Arc;

use crate::{
    commands::danser::BlacklistServerRemove,
    core::Context,
    util::{builder::MessageBuilder, interaction::InteractionCommand, InteractionCommandExt},
};
use eyre::Result;

pub async fn remove(
    ctx: Arc<Context>,
    command: InteractionCommand,
    args: BlacklistServerRemove,
) -> Result<()> {
    let BlacklistServerRemove { guild_id } = args;

    let guild_id_parsed = match guild_id.parse::<u64>() {
        Ok(guild_id) => guild_id,
        Err(_) => {
            command
                .error_callback(&ctx, "Guild ID is invalid!", true)
                .await?;
            return Ok(());
        }
    };

    if ctx.psql().whitelist_server(guild_id_parsed).await? {
        let builder =
            MessageBuilder::new().embed(format!("Server `{guild_id}` successfully whitelisted"));
        command.callback(&ctx, builder, false).await?;
    } else {
        command
            .error_callback(&ctx, format!("Server `{guild_id}` not found"), true)
            .await?;
    }

    Ok(())
}
