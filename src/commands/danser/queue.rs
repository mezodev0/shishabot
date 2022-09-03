use std::{fmt::Write, sync::Arc};

use command_macros::SlashCommand;
use eyre::Result;
use time::OffsetDateTime;
use twilight_interactions::command::{CommandModel, CreateCommand};

use crate::{
    core::{Context, ReplayStatus},
    util::{
        builder::{EmbedBuilder, MessageBuilder},
        interaction::InteractionCommand,
        InteractionCommandExt,
    },
};

#[derive(CreateCommand, CommandModel, SlashCommand)]
#[command(name = "queue")]
#[flags(SKIP_DEFER)]
/// Displays the current replay queue
pub struct Queue;

async fn slash_queue(ctx: Arc<Context>, command: InteractionCommand) -> Result<()> {
    let queue_guard = ctx.replay_queue.queue.lock().await;

    let queue_list = if queue_guard.is_empty() {
        "The queue is empty".to_owned()
    } else {
        let mut s = String::new();
        let status = *ctx.replay_queue.status.lock().await;

        for (replay_data, idx) in queue_guard.iter().zip(1..) {
            let name = replay_data
                .path
                .file_name()
                .expect("missing file name")
                .to_string_lossy();

            let extension = name.rfind(".osr").unwrap_or(name.len());
            let name = name[..extension].replace('_', " ");

            let status = (idx == 1)
                .then_some(status)
                .unwrap_or(ReplayStatus::Waiting);

            let user = replay_data.user;
            let _ = writeln!(s, "{idx}. {name} queued by <@{user}> - {status}");
        }

        s
    };

    let embed = EmbedBuilder::new()
        .title("Current queue")
        .description(queue_list)
        .timestamp(OffsetDateTime::now_utc());

    let builder = MessageBuilder::new().embed(embed);
    command.callback(&ctx, builder, false).await?;

    Ok(())
}
