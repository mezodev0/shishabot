use std::{fmt::Write, sync::Arc};

use command_macros::{command, SlashCommand};
use eyre::Result;
use time::OffsetDateTime;
use twilight_interactions::command::{CommandModel, CreateCommand};

use crate::{
    core::{commands::CommandOrigin, Context, ReplayStatus},
    util::{
        builder::{EmbedBuilder, MessageBuilder},
        interaction::InteractionCommand,
    },
};

#[derive(CreateCommand, CommandModel, SlashCommand)]
#[command(name = "queue")]
#[flags(SKIP_DEFER)]
/// Displays the current replay queue
pub struct Queue;

async fn slash_queue(ctx: Arc<Context>, mut command: InteractionCommand) -> Result<()> {
    queue(ctx, (&mut command).into()).await
}

#[command]
#[desc("Display the current replay queue")]
#[alias("q")]
#[flags(SKIP_DEFER)]
#[group(Danser)]
async fn prefix_queue(ctx: Arc<Context>, msg: &Message) -> Result<()> {
    queue(ctx, msg.into()).await
}

async fn queue(ctx: Arc<Context>, orig: CommandOrigin<'_>) -> Result<()> {
    let queue_guard = ctx.replay_queue.queue.lock().await;

    let queue_list = if queue_guard.is_empty() {
        "The queue is empty".to_owned()
    } else {
        let mut s = String::new();
        let status = *ctx.replay_queue.status.lock().await;

        for (replay_data, idx) in queue_guard.iter().zip(1..) {
            let name = &replay_data
                .path
                // TODO
                // .replace("../Downloads/", "")
                // .replace('_', " ")
                // .replace(".osr", "");
                ;

            let status = (idx == 1)
                .then_some(status)
                .unwrap_or(ReplayStatus::Waiting);

            let user = replay_data.user;
            let _ = writeln!(s, "{idx}. {name:?} queued by <@{user}> - {status}");
        }

        s
    };

    let embed = EmbedBuilder::new()
        .title("Current queue")
        .description(queue_list)
        .timestamp(OffsetDateTime::now_utc());

    let builder = MessageBuilder::new().embed(embed);
    orig.callback(&ctx, builder).await?;

    Ok(())
}
