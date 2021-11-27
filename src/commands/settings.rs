use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
    prelude::*,
};
use tokio::fs;

#[command]
async fn settings(ctx: &Context, msg: &Message) -> CommandResult {
    let author = msg.author.id;
    if !path_exists(format!("../danser/settings/{}.json", author)).await {
        if let Err(why) = fs::copy(
            "../danser/settings/default.json",
            format!("../danser/settings/{}.json", author),
        )
        .await
        {
            println!("Failed to create settings file: {}", why);
        }
    }
    Ok(())
}

async fn path_exists(path: String) -> bool {
    fs::metadata(path).await.is_ok()
}
