use anyhow::Error;
use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::{Message, ReactionType},
    prelude::Context,
};

use crate::process_replays::{parse_attachment_replay, AttachmentParseSuccess, TimePoints};

#[command]
#[description = "**Requires Replay Attachment**\nAllows you to end a replay early"]
#[usage = "[end-time]"]
#[example = "1:30"]
#[example = "90"]
async fn end(ctx: &Context, msg: &Message) -> CommandResult {
    let mut iter = msg.content.split(' ').skip(1).map(TimePoints::parse_single);

    let time_points = match iter.next() {
        Some(Ok(end)) => TimePoints {
            start: None,
            end: Some(end),
        },
        Some(Err(content)) => {
            msg.reply(&ctx, content).await?;

            return Ok(());
        }
        None => {
            msg.reply(&ctx, "You must enter the `[end-time]`!").await?;

            return Ok(());
        }
    };

    match parse_attachment_replay(msg, &ctx.data, Some(time_points)).await {
        Ok(AttachmentParseSuccess::NothingToDo) => {}
        Ok(AttachmentParseSuccess::BeingProcessed) => {
            let reaction = ReactionType::Unicode("✅".to_string());

            if let Err(why) = msg.react(&ctx, reaction).await {
                let err = Error::new(why).context("failed to react after attachment parse success");
                warn!("{err:?}");
            }
        }
        Err(why) => {
            let err = Error::new(why).context("failed to parse attachment");
            warn!("{err:?}");

            if let Err(why) = msg.reply(&ctx, "something went wrong, blame mezo").await {
                let err = Error::new(why).context("failed to reply after attachment parse error");
                warn!("{err:?}");
            }
        }
    }
    Ok(())
}
