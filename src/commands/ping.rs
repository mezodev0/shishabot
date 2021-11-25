use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
    prelude::*,
};

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;
    Ok(())
}
