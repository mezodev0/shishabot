use std::time::Instant;

use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
    prelude::*,
};

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let start = Instant::now();

    let mut response = msg
        .channel_id
        .send_message(ctx, |m| m.content("Pinging..."))
        .await?;

    let elapsed = start.elapsed().as_millis();
    let content = format!("{elapsed}ms");
    response.edit(ctx, |m| m.content(content)).await?;

    Ok(())
}
