use serenity::{
    async_trait,
    framework::standard::{macros::group, StandardFramework},
    model::prelude::*,
    prelude::*,
};
use std::env;
mod commands;
use commands::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name)
    }
}

#[group]
#[commands(ping)]
struct General;

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to read .env file");
    let token = env::var("DISCORD_TOKEN").expect("Expected a token from the env");

    let framework = StandardFramework::new()
        .configure(|c| c.with_whitespace(true).prefix("!!"))
        .group(&GENERAL_GROUP)
        .help(&HELP);

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Failed to create client");

    if let Err(why) = client.start().await {
        println!("Client Error: {:?}", why);
    }
}
