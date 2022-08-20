use anyhow::Error;
use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::{Message, ReactionType},
    prelude::Context,
};

use crate::process_replays::{parse_attachment_replay, AttachmentParseSuccess};

#[command]
#[description = "**Requires Replay Attachment**\nAllows you to end a replay early"]
#[usage = "[end-time]"]
#[example = "1:30"]
#[example = "90"]
async fn end(ctx: &Context, msg: &Message) -> CommandResult {
    let mut times: Vec<&str> = msg.content.split(' ').collect();
    times.remove(0);

    if times.is_empty() {
        msg.reply(&ctx, "You must enter the `[end-time]`!").await?;
        return Ok(());
    }
    let mut times_vec: Vec<String> = Vec::new();
    times_vec.push("0".to_string());

    let time_item = times[0];

    if time_item.contains(':') {
        let time_item_vec: Vec<&str> = time_item.split(':').collect();

        let minutes = time_item_vec[0];
        let seconds = time_item_vec[1];

        match (minutes.parse::<u32>(), seconds.parse::<u32>()) {
            (Ok(minutes), Ok(seconds @ 0..=59)) => {
                let final_time = minutes * 60 + seconds;
                times_vec.push(final_time.to_string());
            }
            _ => {
                msg.reply(&ctx, "A value you supplied is not a number!")
                    .await?;
                return Ok(());
            }
        }
    } else {
        match time_item.parse::<u32>() {
            Ok(unit) => {
                times_vec.push(unit.to_string());
            }
            _ => {
                msg.reply(&ctx, "A value you supplied is not a number!")
                    .await?;
                return Ok(());
            }
        }
    }

    match parse_attachment_replay(msg, &ctx.data, Some(times_vec)).await {
        Ok(AttachmentParseSuccess::NothingToDo) => {}
        Ok(AttachmentParseSuccess::BeingProcessed) => {
            let reaction = ReactionType::Unicode("✅".to_string());
            if let Err(why) = msg.react(&ctx, reaction).await {
                let err = Error::new(why).context("failed to react after attachment parse success");
                warn!("{:?}", err);
            }
        }
        Err(why) => {
            let err = Error::new(why).context("failed to parse attachment");
            warn!("{:?}", err);

            if let Err(why) = msg.reply(&ctx, "something went wrong, blame mezo").await {
                let err = Error::new(why).context("failed to reply after attachment parse error");
                warn!("{:?}", err);
            }
        }
    }
    Ok(())
}
