use anyhow::{Context, Result};
use serenity::{
    builder::ParseValue,
    client::Context as SerenityContext,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
    utils::Colour,
};
use tokio::fs;

use crate::commands::Settings;

#[command]
#[description = "Creates your very own settings file for you to customize!"]
async fn settings(ctx: &SerenityContext, msg: &Message) -> CommandResult {
    let author = msg.author.id;
    let from = "../danser/settings/default.json";
    let to = format!("../danser/settings/{}.json", author);

    if !path_exists(format!("../danser/settings/{}.json", author)).await {
        fs::copy(from, to)
            .await
            .context("failed to create settings file")?;
    }

    let settings_path = format!("../danser/settings/{}.json", author);
    let file_content = tokio::fs::read_to_string(settings_path).await?;
    let settings: Settings = serde_json::from_str(&file_content)?;
    let color = get_user_role_color(&msg, &ctx).await?;

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
                        settings.skin.current_skin,
                        settings.skin.cursor.scale,
                        if settings.cursor.cursor_ripples {
                            "on"
                        } else {
                            "off"
                        },
                        if settings.playfield.background.load_storyboards {
                            "on"
                        } else {
                            "off"
                        },
                        if settings.playfield.background.load_videos {
                            "on"
                        } else {
                            "off"
                        },
                        settings.playfield.background.dim.normal,
                        (settings.audio.music_volume * 100.0),
                        (settings.audio.sample_volume * 100.0),
                        settings.gameplay.pp_counter.decimals,
                        settings.gameplay.hit_error_meter.unstable_rate_decimals,
                        if settings.gameplay.aim_error_meter.show {
                            "on"
                        } else {
                            "off"
                        },
                        settings.gameplay.aim_error_meter.unstable_rate_decimals,
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

async fn get_user_role_color(msg: &Message, ctx: &SerenityContext) -> Result<Colour> {
    let mut roles = msg
        .member(&ctx)
        .await
        .with_context(|| {
            format!(
                "failed to get member {} in guild {}",
                msg.author.id,
                msg.guild_id.unwrap_or_default()
            )
        })?
        .roles(&ctx)
        .await
        .with_context(|| {
            format!(
                "failed to get roles for member {} in guild {}",
                msg.author.id,
                msg.guild_id.unwrap_or_default()
            )
        })?;

    roles.sort_by(|a, b| b.position.cmp(&a.position));

    let color = if let Some(role) = roles.get(0) {
        role.colour
    } else {
        Colour::from_rgb(246, 219, 216)
    };

    Ok(color)
}
