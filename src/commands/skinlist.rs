use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
    prelude::*,
};
use tokio::fs;

#[command]
#[description = "Displays all skins available"]
async fn skinlist(ctx: &Context, msg: &Message) -> CommandResult {
    let mut skins = fs::read_dir("../Skins/").await?;
    let mut counter: i32 = 0;
    let mut skinlist: String = String::from("");

    while let Some(skin) = skins.next_entry().await.unwrap() {
        counter += 1;
        skinlist.push_str(&format!(
            "{}) {}\n",
            counter,
            skin.file_name().into_string().unwrap()
        ));
    }
    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| e.title("Skinlist").description(skinlist))
        })
        .await?;
    Ok(())
}
