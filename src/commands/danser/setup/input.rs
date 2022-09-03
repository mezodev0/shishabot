use std::sync::Arc;

use eyre::Result;
use twilight_model::guild::Permissions;

use crate::{
    core::Context,
    util::{
        builder::MessageBuilder, interaction::InteractionCommand, Authored, InteractionCommandExt,
    },
};

use super::{InputAction, SetupInput};

pub async fn input(ctx: Arc<Context>, command: InteractionCommand, args: SetupInput) -> Result<()> {
    let member = command.member.as_ref().unwrap();
    let permissions = member.permissions.unwrap_or_else(Permissions::empty);

    if permissions.contains(Permissions::ADMINISTRATOR) {
        let guild_id = command.guild_id.unwrap();
        let SetupInput { action, channel } = args;

        match action {
            InputAction::Add => {
                let upsert_res =
                    ctx.upsert_guild_settings(guild_id, |s| s.input_channels.insert(channel));

                if let Err(err) = upsert_res {
                    let content = "Failed to update server settings";
                    let _ = command.error_callback(&ctx, content, false).await;

                    return Err(err);
                }

                let content = format!("Successfully added channel <#{channel}>");
                let builder = MessageBuilder::new().embed(content);
                command.callback(&ctx, builder, false).await?;
            }
            InputAction::Remove => {
                let upsert_res =
                    ctx.upsert_guild_settings(guild_id, |s| s.input_channels.remove(&channel));

                match upsert_res {
                    Ok(true) => {
                        let content = format!("Successfully removed channel <#{channel}>");
                        let builder = MessageBuilder::new().embed(content);
                        command.callback(&ctx, builder, false).await?;
                    }
                    Ok(false) => {
                        let content = "That channel is not whitelisted";
                        command.error_callback(&ctx, content, false).await?;
                    }
                    Err(err) => {
                        let content = "Failed to update server settings";
                        let _ = command.error_callback(&ctx, content, false).await;

                        return Err(err);
                    }
                }
            }
        }
    } else {
        let content = "You do not have the required permissions to perform this action!";
        command.error_callback(&ctx, content, true).await;
    }

    Ok(())
}
