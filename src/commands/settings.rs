use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
    prelude::*,
};
use tokio::{
    fs::{self, File},
    io::AsyncReadExt,
};

#[command]
#[description = "Creates your very own settings file for you to customize!"]
async fn settings(ctx: &Context, msg: &Message) -> CommandResult {
    let author = msg.author.id;
    let from = "../danser/settings/default.json";
    let to = format!("../danser/settings/{}.json", author);

    if !path_exists(format!("../danser/settings/{}.json", author)).await {
        if let Err(why) = fs::copy(from, to).await {
            println!("Failed to create settings file: {}", why);
        }
    }

    let settings_path = format!("../danser/settings/{}.json", author);
    let mut settings_file = File::open(settings_path).await?;
    let mut content = String::new();
    settings_file.read_to_string(&mut content).await?;

    let json: serde_json::Value =
        serde_json::from_str(&content).expect("JSON was not well-formatted");

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title("Settings")
                    .description(format!("{}", json["Skin"][0]["CurrentSkin"]))
            })
        })
        .await?;

    Ok(())
}

async fn path_exists(path: String) -> bool {
    fs::metadata(path).await.is_ok()
}
