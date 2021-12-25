use serenity::{
    builder::ParseValue,
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
    utils::Colour,
};
use tokio::fs::{self};

use crate::commands::Settings;

#[command]
#[description = "Creates your very own settings file for you to customize!"]
async fn settings(ctx: &Context, msg: &Message) -> CommandResult {
    let author = msg.author.id;
    let from = "../danser/settings/default.json";
    let to = format!("../danser/settings/{}.json", author);

    if !path_exists(format!("../danser/settings/{}.json", author)).await {
        if let Err(why) = fs::copy(from, to).await {
            warn!("Failed to create settings file: {}", why);
        }
    }

    let settings_path = format!("../danser/settings/{}.json", author);
    let file_content = tokio::fs::read_to_string(settings_path).await?;
    let settings: Settings = serde_json::from_str(&file_content)?;

    let color = get_user_role_color(&msg, &ctx).await;

    msg.channel_id
        .send_message(&ctx, |m| {
            m.reference_message((msg.channel_id, msg.id))
                .allowed_mentions(|f| {
                    f.replied_user(false)
                        .parse(ParseValue::Everyone)
                        .parse(ParseValue::Users)
                        .parse(ParseValue::Roles)
                });
            m.embed(|e| {
                e.title(format!("Settings for {}", msg.author.name))
                    .description(format!(
                        "**Skin**\n`skin`: {}\n\n**Cursor**\n`cursor size`: {}\n`cursor ripple`: {}\n\n**Background**\n`storyboard`: {}\n`background video`: {}\n`dim`: {}\n\n**Audio**\n`music volume`: {}%\n`hitsound volume`: {}%\n\n**Gameplay**\n`pp counter decimals`: {}\n`hit error decimals`: {}\n`aim error meter`: {}\n`aim error meter ur decimals`: {}",
                        settings.skin.currentSkin,
                        settings.skin.cursor.scale,
                        if settings.cursor.cursorRipples {
                            "on"
                        } else {
                            "off"
                        },
                        if settings.playfield.background.loadStoryboards {
                            "on"
                        } else {
                            "off"
                        },
                        if settings.playfield.background.loadVideos {
                            "on"
                        } else {
                            "off"
                        },
                        settings.playfield.background.dim.normal,
                        (settings.audio.musicVolume * 100.0),
                        (settings.audio.sampleVolume * 100.0),
                        settings.gameplay.ppCounter.decimals,
                        settings.gameplay.hitErrorMeter.unstableRateDecimals,
                        if settings.gameplay.aimErrorMeter.show {
                            "on"
                        } else {
                            "off"
                        },
                        settings.gameplay.aimErrorMeter.unstableRateDecimals,
                    ))
                    .color(color)
            })
        })
        .await?;
    Ok(())
}

async fn path_exists(path: String) -> bool {
    fs::metadata(path).await.is_ok()
}

async fn get_user_role_color(msg: &Message, ctx: &Context) -> Colour {
    let mut roles = msg.member(&ctx).await.unwrap().roles(&ctx).await.unwrap();

    roles.sort_by(|a, b| b.position.cmp(&a.position));

    if roles.len() == 0 {
        return Colour::from_rgb(246, 219, 216);
    }

    return roles[0].colour;
}
