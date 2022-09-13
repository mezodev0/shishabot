use std::sync::Arc;

use command_macros::msg_command;

use crate::{
    core::Context,
    util::{builder::MessageBuilder, interaction::InteractionCommand, InteractionCommandExt},
};

#[msg_command(name = "Command name", dm_permission = false)]
async fn mytest(ctx: Arc<Context>, command: InteractionCommand) -> Result<()> {
    let builder = MessageBuilder::new().embed("great success :thumbsup:");
    command.update(&ctx, &builder).await?;

    Ok(())
}
