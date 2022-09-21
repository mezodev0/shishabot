use std::sync::Arc;

use twilight_model::channel::Message;

use crate::{core::Context, util::ChannelExt};

pub async fn handle_message(ctx: Arc<Context>, msg: Message) {
    if let Some(attachment) = msg.attachments.first() {
        let content = "Hey! Looks like you tried to send a replay\nPlease use **/render** as we have fully migrated to slash commands.";
        if matches!(attachment.filename.split('.').last(), Some("osr")) {
            let valid_input_channel = msg
                .guild_id
                .map(|f| ctx.guild_settings(f, |s| s.input_channels.contains(&msg.channel_id)));

            match valid_input_channel {
                Some(Some(true)) => {
                    let _ = msg.error(&ctx, content).await;
                }
                Some(Some(false) | None) => {}
                None => {
                    let _ = msg.error(&ctx, content).await;
                }
            }
        }
    }
}
