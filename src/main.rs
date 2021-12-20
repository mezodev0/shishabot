#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

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
use std::{env, fs, sync::Arc};

mod commands;
use commands::*;

mod process_replays;
use process_replays::*;

mod logging;

struct ReplayHandler;
impl TypeMapKey for ReplayHandler {
    type Value = tokio::sync::mpsc::UnboundedSender<Data>;
}

struct Handler;
#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        if create_missing_folders().await.is_ok() {
            info!("created folders");
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let data = ctx.data.read().await;
        let sender = data.get::<ReplayHandler>().unwrap();

        match parse_attachment_replay(&msg, sender).await {
            AttachmentParseResult::NoAttachmentOrReplay => {}
            AttachmentParseResult::BeingProcessed => {
                let reaction = ReactionType::Unicode("✅".to_string());

                if let Err(why) = msg.react(&ctx, reaction).await {
                    warn!("failed to reply: {}", why);
                }
            }
            AttachmentParseResult::FailedDownload(err) => {
                warn!("download failed: {}", err);
                if let Err(why) = msg.reply(&ctx, "something went wrong, blame mezo").await {
                    warn!("failed to reply: {}", why);
                }
            }
            AttachmentParseResult::FailedParsing(err) => {
                warn!("parsing failed: {}", err);
                if let Err(why) = msg.reply(&ctx, "something went wrong, blame mezo").await {
                    warn!("failed to reply: {}", why);
                }
            }
        }
    }
}

#[group]
#[commands(ping)]
struct General;

#[group]
#[commands(settings, skinlist)]
struct Danser;

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to read .env file");
    logging::initialize().expect("Failed to initialize logging");

    let token = env::var("DISCORD_TOKEN").expect("Expected a token from the env");
    let client_id: u64 = env::var("CLIENT_ID")
        .expect("Expected client id from the env")
        .parse()
        .unwrap();
    let client_secret: String =
        env::var("CLIENT_SECRET").expect("Expected client secret from the env");

    let framework = StandardFramework::new()
        .configure(|c| c.with_whitespace(true).prefix("!!"))
        .before(log_command)
        .after(finished_command)
        .group(&GENERAL_GROUP)
        .group(&DANSER_GROUP)
        .help(&HELP);

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Failed to create client");

    let http = Arc::clone(&client.cache_and_http.http);

    let osu: Osu = match Osu::new(client_id, client_secret).await {
        Ok(client) => client,
        Err(why) => panic!(
            "Failed to create client or make initial osu!api interaction: {}",
            why
        ),
    };

    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(process_replay(receiver, osu, http));
    {
        let mut data = client.data.write().await;
        data.insert::<ReplayHandler>(sender);
    }

    if let Err(why) = client.start().await {
        error!("Client Error: {:?}", why);
    }
}

async fn create_missing_folders() -> std::io::Result<()> {
    fs::create_dir_all("../Songs")?;
    fs::create_dir_all("../Skins")?;
    fs::create_dir_all("../Downloads")?;
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
        Err(why) => warn!("Command '{}' returned error {:?}", cmd_name, why),
    }
}
