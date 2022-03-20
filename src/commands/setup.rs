use anyhow::{Context, Error};
use serenity::{
    client::Context as SerenityContext,
    framework::standard::{macros::command, CommandResult},
    model::{channel::Message, id::ChannelId},
};

use crate::{commands::server_settings_struct::Server, ServerSettings};

#[command]
#[required_permissions("ADMINISTRATOR")]
#[description = "Setup the input and output channels for your server!"]
async fn setup(ctx: &SerenityContext, msg: &Message) -> CommandResult {
    let mut mentioned_channels = msg
        .content
        .split_whitespace()
        .filter_map(|arg| arg.parse::<ChannelId>().ok());

    if let (Some(id1), Some(id2)) = (mentioned_channels.next(), mentioned_channels.next()) {
        let guild_id = msg.guild_id.unwrap_or_default();

        let edited_settings = {
            let mut data = ctx.data.write().await;
            let settings = data.get_mut::<ServerSettings>().unwrap();

            settings
                .servers
                .entry(guild_id)
                .and_modify(|server| {
                    server.replay_channel = id1;
                    server.output_channel = id2;
                })
                .or_insert_with(|| Server {
                    replay_channel: id1,
                    output_channel: id2,
                    prefixes: Vec::new(),
                });

            serde_json::to_string(settings).context("failed to serialize server settings")?
        };

        if let Err(why) = tokio::fs::write("src/server_settings.json", edited_settings).await {
            let err = Error::new(why).context("failed to edit server specific settings");
            warn!("{:?}", err);
        }

        msg.reply(&ctx, "Successfully changed settings!").await?;
    } else {
        msg.reply(&ctx, "You need to mention 2 channels!").await?;
    }

    Ok(())
}
