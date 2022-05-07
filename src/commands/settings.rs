use anyhow::{Context, Error, Result};
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
    let mut settings: Settings = serde_json::from_str(&file_content)?;
    let color = get_user_role_color(msg, ctx).await?;
    if msg.content.split(" ").count() == 3 {
        let new_settings = msg.content.split(" ").collect::<Vec<&str>>();

        match edit_setting(&mut settings, &new_settings[1], &new_settings[2], &msg).await {
            Ok(_) => {
                msg.reply(&ctx, "Edited setting successfully!").await?;
            }
            Err(EditSettingsError::Other(err)) => {
                let _ = msg.reply(&ctx, "something went wrong, blame mezo").await;

                return Err(err.into());
            }
            Err(why) => {
                msg.reply(&ctx, why).await?;
            }
        }

        return Ok(());
    }

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
                        "**Skin**\n`skin`: {}\n\n\
                        **Cursor**\n`cursor size`: {}\n`cursor ripple`: {}\n\n\
                        **Background**\n`storyboard`: {}\n`background video`: {}\n`dim`: {}\n\n\
                        **Audio**\n`music volume`: {}%\n`hitsound volume`: {}%\n\n\
                        **PP Counter**\n`pp counter decimals`: {}\n\n\
                        **Hit Error Meter**\n`hit error decimals`: {}\n\n\
                        **Aim Error Meter**\n`show aim error meter`: {}\n`aim error meter ur decimals`: {}\n\n\
                        **Hit Counter**\n`show hit counter`: {}\n`show sliderbreaks`: {}",
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
                        if settings.gameplay.hit_counter.show {
                            "on"
                        } else {
                            "off"
                        },
                        if settings.gameplay.hit_counter.show_sliderbreaks {
                            "on"
                        } else {
                            "off"
                        }
                    ))
                    .color(color)
                    .footer(|f| f.text("To edit your settings type !!settings [setting] [value] | The setting name is the same as in the embed, spaces are replaced with '_'"))
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

    roles.sort_unstable_by_key(|role| -role.position);

    let color = if let Some(role) = roles.get(0) {
        role.colour
    } else {
        Colour::from_rgb(246, 219, 216)
    };

    Ok(color)
}

#[derive(Debug, thiserror::Error)]
enum EditSettingsError {
    #[error("Aim error meter ur decimals have to be between 1 and 3!")]
    InvalidAimErrorDecimals,
    #[error("Cursorsize has to be between 0.1 and 2!")]
    InvalidCursorSize,
    #[error("Dim has to be between 0 and 1!")]
    InvalidDim,
    #[error("Hit error decimals have to be between 1 and 3!")]
    InvalidHitErrorDecimals,
    #[error("Hitsound volume has to be between 1 and 100!")]
    InvalidHitsoundVolume,
    #[error("Music volume has to be between 1 and 100!")]
    InvalidMusicVolume,
    #[error("PP counter decimals have to be between 1 and 3!")]
    InvalidPpCounterDecimals,
    #[error("Skin is not valid!")]
    InvalidSkin,
    #[error("Value is not valid!")]
    InvalidValue,
    #[error("Couldn't find skin!")]
    MissingSkin,
    #[error(transparent)]
    Other(#[from] Error),
}

async fn edit_setting(
    settings: &mut Settings,
    key: &str,
    value: &str,
    msg: &Message,
) -> Result<(), EditSettingsError> {
    match key {
        "skin" => {
            let value_as_number: i32 = value.parse().map_err(|_| EditSettingsError::InvalidSkin)?;

            let mut skins = fs::read_dir("../Skins/")
                .await
                .context("failed to read dir `../Skins/`")?;

            let mut counter = 0;
            let mut skin_found = false;

            while let Some(skin) = skins
                .next_entry()
                .await
                .context("failed to get entry of `../Skins/`")?
            {
                let file_name = skin.file_name();
                counter += 1;

                if counter == value_as_number {
                    settings.skin.current_skin = file_name.into_string().unwrap();
                    skin_found = true;
                    break;
                }
            }

            if !skin_found {
                return Err(EditSettingsError::MissingSkin);
            }
        }
        "cursor_size" => {
            let value_as_number: f64 =
                value.parse().map_err(|_| EditSettingsError::InvalidValue)?;

            if value_as_number < 0.1 || value_as_number > 2.0 {
                return Err(EditSettingsError::InvalidCursorSize);
            }

            settings.skin.cursor.scale = value_as_number;
        }
        "cursor_ripple" => {
            settings.cursor.cursor_ripples =
                matches!(value.to_uppercase().as_str(), "ON" | "TRUE" | "YES");
        }
        "storyboard" => {
            settings.playfield.background.load_storyboards =
                matches!(value.to_uppercase().as_str(), "ON" | "TRUE" | "YES");
        }
        "background_video" | "video" => {
            settings.playfield.background.load_videos =
                matches!(value.to_uppercase().as_str(), "ON" | "TRUE" | "YES");
        }
        "dim" => {
            let value_as_number: f64 =
                value.parse().map_err(|_| EditSettingsError::InvalidValue)?;

            if value_as_number < 0.0 || value_as_number > 1.0 {
                return Err(EditSettingsError::InvalidDim);
            }

            settings.playfield.background.dim.normal = value_as_number;
        }
        "music_volume" | "music" => {
            let value_as_number: f64 = value
                .trim_end_matches('%')
                .parse()
                .map_err(|_| EditSettingsError::InvalidValue)?;

            if value_as_number < 1.0 || value_as_number > 100.0 {
                return Err(EditSettingsError::InvalidMusicVolume);
            }

            settings.audio.music_volume = value_as_number / 100.0;
        }
        "hitsound_volume" | "hitsound" => {
            let value_as_number: f64 = value
                .trim_end_matches('%')
                .parse()
                .map_err(|_| EditSettingsError::InvalidValue)?;

            if value_as_number < 1.0 || value_as_number > 100.0 {
                return Err(EditSettingsError::InvalidHitsoundVolume);
            }

            settings.audio.sample_volume = value_as_number / 100.0;
        }
        "pp_counter_decimals" => {
            let value_as_number: u64 =
                value.parse().map_err(|_| EditSettingsError::InvalidValue)?;

            if value_as_number < 1 || value_as_number > 3 {
                return Err(EditSettingsError::InvalidPpCounterDecimals);
            }

            settings.gameplay.pp_counter.decimals = value_as_number;
        }
        "hit_error_decimals" => {
            let value_as_number: u64 =
                value.parse().map_err(|_| EditSettingsError::InvalidValue)?;

            if value_as_number < 1 || value_as_number > 3 {
                return Err(EditSettingsError::InvalidHitErrorDecimals);
            }

            settings.gameplay.pp_counter.decimals = value_as_number;
        }
        "aim_error_meter_ur_decimals" => {
            let value_as_number: u64 =
                value.parse().map_err(|_| EditSettingsError::InvalidValue)?;

            if value_as_number < 1 || value_as_number > 3 {
                return Err(EditSettingsError::InvalidAimErrorDecimals);
            }

            settings.gameplay.pp_counter.decimals = value_as_number;
        }
        "show_aim_error_meter" | "aim_error_meter" => {
            settings.gameplay.aim_error_meter.show =
                matches!(value.to_uppercase().as_str(), "ON" | "TRUE" | "YES");
        }
        "show_hit_counter" | "hit_counter" => {
            settings.gameplay.hit_counter.show =
                matches!(value.to_uppercase().as_str(), "ON" | "TRUE" | "YES");
        }
        "show_sliderbreaks" => {
            settings.gameplay.hit_counter.show =
                matches!(value.to_uppercase().as_str(), "ON" | "TRUE" | "YES");
        }
        _ => {}
    }

    let edited_setting =
        serde_json::to_string(&settings).context("failed to serialize edited settings")?;

    let path = format!("../danser/settings/{}.json", msg.author.id);

    tokio::fs::write(path, edited_setting)
        .await
        .with_context(|| {
            format!(
                "failed writing to `../danser/settings/{}.json` on edit_setting",
                msg.author.id
            )
        })?;

    Ok(())
}
