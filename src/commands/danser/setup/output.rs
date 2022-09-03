use std::sync::Arc;

use eyre::Result;
use twilight_model::guild::Permissions;

use crate::{
    core::Context,
    util::{
        builder::MessageBuilder, interaction::InteractionCommand, Authored, InteractionCommandExt,
    },
};

use super::SetupOutput;

pub async fn output(
    ctx: Arc<Context>,
    command: InteractionCommand,
    args: SetupOutput,
) -> Result<()> {
    let member = command.member.as_ref().unwrap();
    let permissions = member.permissions.unwrap_or_else(Permissions::empty);

    if permissions.contains(Permissions::ADMINISTRATOR) {
        let guild_id = command.guild_id.unwrap();
        let SetupOutput { channel } = args;

        let upsert_res = ctx.upsert_guild_settings(guild_id, |s| s.output_channel = Some(channel));

        if let Err(err) = upsert_res {
            let content = "Failed to update server settings";
            let _ = command.error_callback(&ctx, content, false).await;

            return Err(err);
        }

        let content = format!("Successfully specified <#{channel}> as output");
        let builder = MessageBuilder::new().embed(content);
        command.callback(&ctx, builder, false).await;
    } else {
        let content = "You do not have the required permissions to perform this action!";
        command.error_callback(&ctx, content, true).await?;
    }

    Ok(())
}
