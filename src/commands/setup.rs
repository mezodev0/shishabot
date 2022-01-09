use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::{channel::Message, id::ChannelId},
    prelude::*,
};

use crate::commands::server_settings_struct::Root;

#[command]
#[required_permissions("ADMINISTRATOR")]
#[description = "Setup the input and output channels for your server!"]
async fn setup(ctx: &Context, msg: &Message) -> CommandResult {
    let mut mentioned_channels = msg
        .content
        .split_whitespace()
        .filter_map(|arg| arg.parse::<ChannelId>().ok());

    if let (Some(id1), Some(id2)) = (mentioned_channels.next(), mentioned_channels.next()) {
        let settings_file = tokio::fs::read_to_string("src/server_settings.json").await?;
        let mut settings: Root = serde_json::from_str(&settings_file)?;
        let mut count = 0;
        for mut server in settings.servers.iter_mut() {
            if server.server_id == msg.guild_id.unwrap().to_string() {
                server.replay_channel = id1.to_string();
                server.output_channel = id2.to_string();
                count += 1;
                break;
            }
        }

        let edited_settings = serde_json::to_string(&settings).unwrap();
        if let Err(why) = tokio::fs::write("src/server_settings.json", edited_settings).await {
            warn!("Failed to edit server specific settings: {}", why);
        }
        if count == 0 {
            msg.reply(&ctx, "There was an error setting up the channels!")
                .await?;
        } else {
            msg.reply(&ctx, "Successfully changed settings!").await?;
        }
    } else {
        msg.reply(&ctx, "You need to mention 2 channels!").await?;
    };

    Ok(())
}
