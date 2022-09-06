use std::{borrow::Cow, fmt::Write, sync::Arc};

use command_macros::SlashCommand;
use eyre::Result;
use time::OffsetDateTime;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::channel::embed::EmbedField;

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
                "\\🏃‍♂️"
            } else {
                "\\✅"
            },
            rendering = match status {
                ReplayStatus::Downloading => "\\⌛".into(),
                ReplayStatus::Rendering(progress) => Cow::Owned(format!("\\🏃‍♂️ ({progress}%)")),
                _ => "\\✅".into(),
            },
            encoding = match status {
                ReplayStatus::Encoding(progress) => Cow::Owned(format!("\\🏃‍♂️ ({progress}%)")),
                ReplayStatus::Uploading => "\\✅".into(),
                _ => "\\⌛".into(),
            },
            uploading = if let ReplayStatus::Uploading = status {
                "\\🏃‍♂️"
            } else {
                "\\⌛"
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
