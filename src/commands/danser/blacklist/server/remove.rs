use std::sync::Arc;

use crate::{
    commands::danser::BlacklistServerRemove,
    core::Context,
    util::{interaction::InteractionCommand, InteractionCommandExt},
};
use eyre::Result;

pub async fn remove(
    ctx: Arc<Context>,
    command: InteractionCommand,
    _args: BlacklistServerRemove,
) -> Result<()> {
    command.error_callback(&ctx, "!!!!", false).await?;
    Ok(())
}
