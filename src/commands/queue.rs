use std::fmt::Write;

use chrono::Utc;
use serenity::{
    builder::ParseValue,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
    prelude::*,
    utils::Color,
};

use crate::ReplayHandler;

#[command]
#[description = "Displays the current replay queue"]
#[aliases("q")]
async fn queue(ctx: &Context, msg: &Message) -> CommandResult {
    let data_guard = ctx.data.read().await;
    let queue_guard = data_guard
        .get::<ReplayHandler>()
        .unwrap()
        .queue
        .lock()
        .await;

    let queue_list = if queue_guard.is_empty() {
        "The queue is empty".to_string()
    } else {
        let mut s = String::new();
        let status = *data_guard
            .get::<ReplayHandler>()
            .unwrap()
            .status
            .lock()
            .await;
        for (replay_data, idx) in queue_guard.iter().zip(1..) {
            let _ = writeln!(
                s,
                "{idx}. {} queued by <@{}> - {}",
                replay_data
                    .path
                    .replace("../Downloads/", "")
                    .replace("_", " ")
                    .replace(".osr", ""),
                replay_data.user,
                if idx == 1 {
                    match status {
                        crate::ReplayStatus::Waiting => "Waiting",
                        crate::ReplayStatus::Downloading => "Downloading",
                        crate::ReplayStatus::Processing => "Processing",
                        crate::ReplayStatus::Uploading => "Uploading",
                    }
                } else {
                    "Waiting"
                }
            );
        }

        s
    };

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
                e.title("Current queue")
                    .description(queue_list)
                    .color(Color::new(15785176))
                    .timestamp(Utc::now())
            })
        })
        .await?;

    Ok(())
}
