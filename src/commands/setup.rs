use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
    prelude::*,
};

use crate::commands::server_settings_struct::Root;

#[command]
async fn setup(ctx: &Context, msg: &Message) -> CommandResult {
    let settings_file = tokio::fs::read_to_string("server_settings.json").await?;
    let settings: Root = serde_json::from_str(&settings_file)?;

    for server in settings.servers {
        if msg.guild_id.unwrap().to_string() == server.server_id {}
    }
    Ok(())
}
