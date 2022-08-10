use std::collections::hash_map::Entry;

use crate::checks::PERMISSIONS_CHECK;
use crate::{server_settings::Server, ServerSettings};
use anyhow::{Context, Error};
use serenity::builder::ParseValue;
use serenity::utils::Color;
use serenity::{
    client::Context as SerenityContext,
    framework::standard::{macros::command, CommandResult},
    model::{channel::Message, id::ChannelId},
};

#[command]
#[checks(Permissions)]
#[only_in(guilds)]
#[description = "Setup the input and output channels for your server"]
#[usage = "[input-channel] [output-channel]"]
#[example = "#channel-1 #channel-2"]
async fn setup(ctx: &SerenityContext, msg: &Message) -> CommandResult {
    let mut mentioned_channels = msg
        .content
        .split_whitespace()
        .filter_map(|arg| arg.parse::<ChannelId>().ok());

    let mut data = ctx.data.write().await;
    let settings = data.get_mut::<ServerSettings>().unwrap();

    if let (Some(id1), Some(id2)) = (mentioned_channels.next(), mentioned_channels.next()) {
        let guild_id = msg.guild_id.unwrap_or_default();

        let edited_settings = {
            settings
                .servers
                .entry(guild_id)
                .and_modify(|server| {
                    server.input_channel = id1;
                    server.output_channel = id2;
                })
                .or_insert_with(|| Server {
                    input_channel: id1,
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
    } else if let Entry::Occupied(o) = settings.servers.entry(msg.guild_id.unwrap_or_default()) {
        if o.get().output_channel != 0 {
            msg.channel_id
                .send_message(&ctx, |m| {
                    m.reference_message((msg.channel_id, msg.id))
                        .allowed_mentions(|f| {
                            f.replied_user(false)
                                .parse(ParseValue::Everyone)
                                .parse(ParseValue::Users)
                                .parse(ParseValue::Roles)
                        });
                    m.embed(|e| {
                        e.title(format!(
                            "Current channel setup{}",
                            if let Some(guild) = msg.guild_id {
                                format!(" for {}", guild.name(&ctx).unwrap_or_default())
                            } else {
                                "".to_owned()
                            }
                        ))
                        .description(format!(
                            "**Input Channel:** <#{}>\n**Output Channel:** <#{}>",
                            o.get().input_channel,
                            o.get().output_channel
                        ))
                        .footer(|f| {
                            {
                                f.text("Use !!setup [input-channel] [output-channel] to edit this")
                            }
                        })
                        .color(Color::new(15785176))
                    })
                })
                .await?;
        }
    } else {
        msg.reply(&ctx, "You need to mention 2 channels!").await?;
    }

    Ok(())
}
