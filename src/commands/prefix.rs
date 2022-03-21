use std::{collections::hash_map::Entry, fmt::Write};

use anyhow::{Context as AnyhowContext, Error};
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::{channel::Message, id::ChannelId},
};

use crate::{commands::server_settings_struct::Server, ServerSettings, DEFAULT_PREFIX};

#[command]
#[description = "Adjust prefixes in a server"]
#[usage = "[add/remove] [space separated list of quoted perfixes]"]
#[example = "add !! \"another prefix\" ~~"]
#[example = "remove ~~"]
#[only_in(guilds)]
#[required_permissions("ADMINISTRATOR")]
async fn prefix(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let mut args = args.raw_quoted();
    let guild_id = msg.guild_id.unwrap();

    match args.next() {
        Some("add" | "a") => {
            let mut data = ctx.data.write().await;
            let settings = data.get_mut::<ServerSettings>().unwrap();

            let prefixes = match settings.servers.entry(guild_id) {
                Entry::Occupied(e) => &mut e.into_mut().prefixes,
                Entry::Vacant(e) => {
                    let server = Server {
                        replay_channel: ChannelId(0),
                        output_channel: ChannelId(0),
                        prefixes: Vec::new(),
                    };

                    &mut e.insert(server).prefixes
                }
            };

            for arg in args {
                if !prefixes.iter().any(|prefix| prefix == arg) {
                    prefixes.push(arg.to_owned());
                }
            }

            let mut prefixes = prefixes.to_owned().into_iter();

            let edited_settings =
                serde_json::to_string(settings).context("failed to serialize server settings")?;

            drop(data);

            if let Err(why) = tokio::fs::write("src/server_settings.json", edited_settings).await {
                let err = Error::new(why).context("failed to edit server specific settings");
                warn!("{:?}", err);
            }

            let mut content = "Successfully added prefixes.\n".to_owned();

            if let Some(prefix) = prefixes.next() {
                let _ = write!(content, "Current prefixes for this server: `{prefix}`");

                for prefix in prefixes {
                    let _ = write!(content, ", `{prefix}`");
                }
            } else {
                let _ = write!(
                    content,
                    "There are no configured prefixes for this server \
                    so only the default prefix `{DEFAULT_PREFIX}` works.",
                );
            }

            msg.channel_id
                .send_message(ctx, |m| m.embed(|e| e.description(content)))
                .await?;

            Ok(())
        }
        Some("remove" | "r") => {
            let mut data = ctx.data.write().await;
            let settings = data.get_mut::<ServerSettings>().unwrap();

            if let Some(entry) = settings.servers.get_mut(&guild_id) {
                let len = entry.prefixes.len();

                for arg in args {
                    entry.prefixes.retain(|prefix| prefix != arg);
                }

                let prefixes = if entry.prefixes.len() != len {
                    let prefixes = entry.prefixes.clone();

                    let edited_settings = serde_json::to_string(settings)
                        .context("failed to serialize server settings")?;

                    drop(data);

                    if let Err(why) =
                        tokio::fs::write("src/server_settings.json", edited_settings).await
                    {
                        let err =
                            Error::new(why).context("failed to edit server specific settings");
                        warn!("{:?}", err);
                    }

                    prefixes
                } else {
                    let prefixes = entry.prefixes.clone();
                    drop(data);

                    prefixes
                };

                let mut content = "Successfully removed prefixes.\n".to_owned();
                let mut prefixes = prefixes.into_iter();

                if let Some(prefix) = prefixes.next() {
                    let _ = write!(content, "Current prefixes for this server: `{prefix}`");

                    for prefix in prefixes {
                        let _ = write!(content, ", `{prefix}`");
                    }
                } else {
                    let _ = write!(
                        content,
                        "There are no configured prefixes for this server \
                        so only the default prefix `{DEFAULT_PREFIX}` works.",
                    );
                }

                msg.channel_id
                    .send_message(ctx, |m| m.embed(|e| e.description(content)))
                    .await?;
            } else {
                let content = format!(
                    "There are no configured prefixes for this server \
                    so only the default prefix `{DEFAULT_PREFIX}` works."
                );

                msg.channel_id
                    .send_message(ctx, |m| m.embed(|e| e.description(content)))
                    .await?;
            }

            Ok(())
        }
        Some(_) => {
            let content = "Either don't provide any arguments to list all current prefixes \
                or use `add` or `remove` as first argument to modify the server prefixes.";

            msg.channel_id.say(ctx, content).await?;

            Ok(())
        }
        None => {
            let data = ctx.data.read().await;
            let settings = data.get::<ServerSettings>().unwrap();

            match settings.servers.get(&guild_id) {
                Some(entry) => {
                    let mut prefixes = entry.prefixes.iter();

                    if let Some(prefix) = prefixes.next() {
                        let mut content = format!("Current prefixes for this server: `{prefix}`");

                        for prefix in prefixes {
                            let _ = write!(content, ", `{prefix}`");
                        }

                        msg.channel_id
                            .send_message(ctx, |m| m.embed(|e| e.description(content)))
                            .await?;

                        Ok(())
                    } else {
                        no_prefix(ctx, msg).await
                    }
                }
                None => no_prefix(ctx, msg).await,
            }
        }
    }
}

async fn no_prefix(ctx: &Context, msg: &Message) -> CommandResult {
    let content = format!(
        "There are no configured prefixes for this server \
        so only the default prefix `{DEFAULT_PREFIX}` works.\n\
        Use `add` or `remove` as first argument to this command to adjust prefixes."
    );

    msg.channel_id
        .send_message(ctx, |m| m.embed(|e| e.description(content)))
        .await?;

    Ok(())
}
