use eyre::Result;
use std::sync::Arc;
use twilight_model::guild::Permissions;

use crate::{
    core::Context,
    util::{
        builder::MessageBuilder, interaction::InteractionCommand, Authored, InteractionCommandExt,
    },
};

use super::SetupInput;

pub async fn input(ctx: Arc<Context>, command: InteractionCommand, args: SetupInput) -> Result<()> {
    let member = command.member.as_ref().unwrap();

    if member
        .permissions
        .unwrap()
        .contains(Permissions::ADMINISTRATOR)
    {
        let SetupInput { action, channel } = args;
        let mut content = String::new();

        match action.value() {
            "add" => {
                ctx.upsert_guild_setings(command.guild_id.unwrap(), |s| {
                    let _ = s.input_channels.insert(channel);
                })
                .await;
                content = format!("Successfully added channel <#{channel}>");
            }
            "remove" => {
                ctx.upsert_guild_setings(
                    command.guild_id.unwrap(),
                    |s: &mut crate::core::settings::Server| {
                        let exists = s.input_channels.remove(&channel);
                        if !exists {
                            content = String::from("That channel is not whitelisted");
                        } else {
                            content = format!("Successfully removed channel <#{channel}>");
                        }
                    },
                )
                .await;
            }
            _ => unreachable!(),
        }

        let builder = MessageBuilder::new().embed(&*content);

        if content.contains("whitelisted") {
            command.error_callback(&ctx, content, false).await;
        } else {
            command.callback(&ctx, builder, false).await;
        }
    } else {
        let content = "You do not have the required permissions to perform this action!";
        command.error_callback(&ctx, content, true).await;
    }

    Ok(())
}
