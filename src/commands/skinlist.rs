use std::fmt::Write;

use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
    prelude::*,
};
use tokio::fs;

use crate::commands::Color;

#[command]
#[description = "Displays all skins available"]
async fn skinlist(ctx: &Context, msg: &Message) -> CommandResult {
    let mut skins = fs::read_dir("../Skins/").await?;
    let mut counter = 0;
    let mut skinlist = String::new();

    while let Some(skin) = skins.next_entry().await? {
        counter += 1;
        let file_name = skin.file_name();
        let _ = writeln!(
            skinlist,
            "{}) {}",
            counter,
            file_name.to_string_lossy().replace("_", " ")
        );
    }

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title("Skinlist")
                    .description(skinlist)
                    .color(Color::new(15785176))
            })
        })
        .await?;

    Ok(())
}
