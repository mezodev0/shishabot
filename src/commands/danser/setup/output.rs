use eyre::Result;
use std::sync::Arc;
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

    if member
        .permissions
        .unwrap()
        .contains(Permissions::ADMINISTRATOR)
    {
        let SetupOutput { channel } = args;
        ctx.upsert_guild_setings(command.guild_id.unwrap(), |s| {
            let _ = s.output_channel = Some(channel);
        })
        .await;

        let content = format!("Successfully specified <#{channel}> as output");
        let builder = MessageBuilder::new().embed(content);
        command.callback(&ctx, builder, false).await;
    } else {
        let content = "You do not have the required permissions to perform this action!";
        command.error_callback(&ctx, content, true).await;
    }

    Ok(())
}
