use std::sync::Arc;

use twilight_model::channel::Message;

use crate::{core::Context, util::ChannelExt};

pub async fn handle_message(ctx: Arc<Context>, msg: Message) {
    match msg.attachments.first() {
        Some(attachment) => {
            if matches!(attachment.filename.split('.').last(), Some("osr")) {
                let content = "Hey! Looks like you tried to send a replay\nPlease use **/render** as we have fully migrated to slash commands.";
                let _ = msg.error(&ctx, content).await;
            }
        }
        None => return,
    }
}
