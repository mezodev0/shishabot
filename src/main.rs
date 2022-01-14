#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

use std::{
    env,
    fs::{self, File},
    io::Write,
    path::Path,
    sync::Arc,
};

use anyhow::{Error, Result};
use rosu_v2::Osu;
use serenity::{
    async_trait,
    framework::standard::{
        macros::{group, hook},
        CommandResult, StandardFramework,
    },
    model::prelude::*,
    prelude::*,
};
use tokio::sync::mpsc;

use crate::commands::server_settings_struct::Server;

mod commands;
use commands::*;

mod process_replays;
use process_replays::*;

mod logging;

struct ReplayHandler;
impl TypeMapKey for ReplayHandler {
    type Value = mpsc::UnboundedSender<Data>;
}

struct Handler;
#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
        ctx.set_activity(Activity::watching("!!help - Waiting for replay"))
            .await;
    }

    async fn guild_create(&self, _ctx: Context, guild: Guild, is_new: bool) {
        if is_new {
            let new_setting: Server = Server {
                server_id: guild.id.to_string(),
                replay_channel: String::new(),
                output_channel: String::new(),
            };

            let settings_file = match tokio::fs::read_to_string("src/server_settings.json").await {
                Ok(content) => content,
                Err(why) => {
                    let err = Error::new(why)
                        .context("failed to read `src/server_settings.json` on GuildCreate");
                    return error!("{:?}", err);
                }
            };

            let mut existing_settings: server_settings_struct::Root =
                match serde_json::from_str(&settings_file) {
                    Ok(settings) => settings,
                    Err(why) => {
                        let err = Error::new(why)
                            .context("failed to deserialize settings file on GuildCreate");
                        return error!("{:?}", err);
                    }
                };

            existing_settings.servers.push(new_setting);

            let final_file = match serde_json::to_string(&existing_settings) {
                Ok(content) => content,
                Err(why) => {
                    let err =
                        Error::new(why).context("failed to serialize settings on GuildCreate");
                    return error!("{:?}", err);
                }
            };

            if let Err(why) = tokio::fs::write("src/server_settings.json", final_file).await {
                let err = Error::new(why).context(format!(
                    "failed writing to `src/server_settings.json` on GuildCreate"
                ));
                warn!("{:?}", err);
            }
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let data = ctx.data.read().await;
        let sender = data.get::<ReplayHandler>().unwrap();

        match parse_attachment_replay(&msg, sender, ctx.shard.clone()).await {
            Ok(AttachmentParseSuccess::NoAttachmentOrReplay) => {}
            Ok(AttachmentParseSuccess::BeingProcessed) => {
                let reaction = ReactionType::Unicode("✅".to_string());
                if let Err(why) = msg.react(&ctx, reaction).await {
                    let err =
                        Error::new(why).context("failed to react after attachment parse success");
                    warn!("{:?}", err);
                }
            }
            Err(why) => {
                let err = Error::new(why).context("failed to parse attachment");
                warn!("{:?}", err);

                if let Err(why) = msg.reply(&ctx, "something went wrong, blame mezo").await {
                    let err =
                        Error::new(why).context("failed to reply after attachment parse error");
                    warn!("{:?}", err);
                }
            }
        }
    }
}

#[group]
#[commands(ping)]
struct General;

#[group]
#[commands(settings, skinlist, setup)]
struct Danser;

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to read .env file");
    logging::initialize().expect("Failed to initialize logging");

    match create_missing_folders_and_files().await {
        Ok(_) => info!("created folders and files"),
        Err(why) => panic!("{:?}", why),
    }

    let token = env::var("DISCORD_TOKEN").expect("Expected a token from the env");

    let client_id: u64 = env::var("CLIENT_ID")
        .expect("Expected client id from the env")
        .parse()
        .expect("Expected client id to be an integer");

    let client_secret: String =
        env::var("CLIENT_SECRET").expect("Expected client secret from the env");

    let framework = StandardFramework::new()
        .configure(|c| c.with_whitespace(true).prefix("!!"))
        .before(log_command)
        .after(finished_command)
        .group(&GENERAL_GROUP)
        .group(&DANSER_GROUP)
        .help(&HELP);

    let client_fut = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework);

    let mut client = match client_fut.await {
        Ok(client) => client,
        Err(why) => panic!(
            "{:?}",
            Error::new(why).context("failed to create discord client")
        ),
    };

    let osu: Osu = match Osu::new(client_id, client_secret).await {
        Ok(client) => client,
        Err(why) => panic!(
            "{:?}",
            Error::new(why).context("failed to create osu! client")
        ),
    };

    let http = Arc::clone(&client.cache_and_http.http);
    let (sender, receiver) = mpsc::unbounded_channel();
    tokio::spawn(process_replay(receiver, osu, http));

    {
        let mut data = client.data.write().await;
        data.insert::<ReplayHandler>(sender);
    }

    if let Err(why) = client.start().await {
        error!("{:?}", Error::new(why).context("critical client error"));
    }

    info!("Shutting down");
}

async fn create_missing_folders_and_files() -> Result<()> {
    use anyhow::Context;

    fs::create_dir_all("../Songs").context("failed to create `../Songs`")?;
    fs::create_dir_all("../Skins").context("failed to create `../Skins`")?;
    fs::create_dir_all("../Replays").context("failed to create `../Replays`")?;
    fs::create_dir_all("../Downloads").context("failed to create `../Downloads`")?;

    if !Path::new("src/server_settings.json").exists() {
        let mut file = File::create("src/server_settings.json")
            .context("failed to create file `src/server_settings.json`")?;
        file.write_all(b"{\"Servers\":[]}")
            .context("failed writing to `src/server_settings.json`")?;
    }

    Ok(())
}

#[hook]
async fn log_command(_: &Context, msg: &Message, cmd_name: &str) -> bool {
    info!("Got command '{}' by user '{}'", cmd_name, msg.author.name);

    true
}

#[hook]
async fn finished_command(_: &Context, _: &Message, cmd_name: &str, cmd_result: CommandResult) {
    match cmd_result {
        Ok(()) => info!("Processed command '{}'", cmd_name),
        Err(why) => {
            warn!("Command '{}' returned error: {}", cmd_name, why);
            let mut e = &*why as &dyn std::error::Error;

            while let Some(src) = e.source() {
                warn!("  - caused by: {}", src);
                e = src;
            }
        }
    }
}
