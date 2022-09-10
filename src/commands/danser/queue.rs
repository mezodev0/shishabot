use std::{
    fmt::{Display, Formatter, Result as FmtResult, Write},
    sync::Arc,
};

use command_macros::SlashCommand;
use eyre::Result;
use time::OffsetDateTime;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::channel::embed::EmbedField;

use crate::{
    core::{BotConfig, Context, ReplayStatus},
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
    let status = *ctx.replay_queue.status.lock().await;

    let mut embed = EmbedBuilder::new()
        .title("Current queue")
        .timestamp(OffsetDateTime::now_utc());

    let mut iter = queue_guard.iter();

    if let Some(data) = iter.next() {
        let name = "Progress".to_owned();

        let value = format!(
            "<@{user}>: {name}\n\
            • Downloading: {downloading}\n\
            • Rendering: {rendering}\n\
            • Encoding: {encoding}\n\
            • Uploading: {uploading}",
            user = data.user,
            name = data.replay_name(),
            downloading = if let ReplayStatus::Downloading = status {
                ProcessStatus::Running(None)
            } else {
                ProcessStatus::Done
            },
            rendering = match status {
                ReplayStatus::Downloading => ProcessStatus::Waiting,
                ReplayStatus::Rendering(progress) => ProcessStatus::Running(Some(progress)),
                _ => ProcessStatus::Done,
            },
            encoding = match status {
                ReplayStatus::Encoding(progress) => ProcessStatus::Running(Some(progress)),
                ReplayStatus::Uploading => ProcessStatus::Done,
                _ => ProcessStatus::Waiting,
            },
            uploading = if let ReplayStatus::Uploading = status {
                ProcessStatus::Running(None)
            } else {
                ProcessStatus::Waiting
            },
        );

        let mut fields = vec![EmbedField {
            inline: false,
            name,
            value,
        }];

        if let Some(data) = iter.next() {
            let name = "Upcoming".to_owned();
            let mut value = String::with_capacity(128);

            let _ = writeln!(value, "`2.` <@{}>: {}", data.user, data.replay_name());

            for (data, idx) in iter.zip(3..) {
                let _ = writeln!(value, "`{idx}.` <@{}>: {}", data.user, data.replay_name());
            }

            fields.push(EmbedField {
                inline: false,
                name,
                value,
            });
        }

        embed = embed.fields(fields);
    } else {
        embed = embed.description("The queue is empty");
    }

    let builder = MessageBuilder::new().embed(embed);
    command.callback(&ctx, builder, false).await?;

    Ok(())
}

enum ProcessStatus {
    Done,
    Running(Option<u8>),
    Waiting,
}

impl Display for ProcessStatus {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ProcessStatus::Done => write!(f, "{}", BotConfig::get().emojis.white_check_mark),
            ProcessStatus::Running(Some(progress)) => {
                write!(f, "{} ({progress}%)", BotConfig::get().emojis.man_running)
            }
            ProcessStatus::Running(None) => write!(f, "{}", BotConfig::get().emojis.man_running),
            ProcessStatus::Waiting => write!(f, "{}", BotConfig::get().emojis.hourglass),
        }
    }
}
