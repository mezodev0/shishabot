#![allow(clippy::manual_range_contains)]

use anyhow::{Context, Error, Result};
use serenity::{
    builder::ParseValue,
    client::Context as SerenityContext,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
    utils::Color,
};
use tokio::fs;

use crate::commands::Settings;

#[command]
#[description = "**Skin**
!!settings skin `[index]` - changes the skin

**Cursor**
!!settings cursor_size `[0.1 - 2.0]` - changes the cursor size
!!settings cursor_ripple `[on/off]` - enable/disable cursor ripple

**Background**
!!settings storyboard `[on/off]` - enable/disable storyboard
!!settings background_video `[on/off]` - enable/disable background video
!!settings dim `[0.0 - 1.0]` - change the background dim

**Audio**
!!settings music_volume `[0 - 100]` - change the music volume
!!settings hitsound_volume `[0 - 100]` - change the hitsound volume
!!settings beatmap_hitsounds `[on/off]` - enable/disable beatmap hitsounds

**PP Counter**
!!settings pp_counter_decimals `[0 - 3]` - changes the amount of decimals displayed on the pp counter

**Hit Error Meter**
!!settings hit_error_decimals `[0 - 3]` - changes the amount of decimals displayed on the hit error meter

**Aim Error Meter**
!!settings show_aim_error_meter `[on/off]` - enable/disable the aim error meter
!!settings aim_error_meter_ur_decimals `[0 - 3]` - changes the amount of ur decimals displayed on the aim error meter

**Hit Counter**
!!settings show_hit_counter `[on/off]` - enable/disable the hit counter
!!settings show_sliderbreaks `[on/off]` - adds/removes a sliderbreak counter to the hit counter

**Strain Graph**
!!settings show_strain_graph`[on/off]` - enable/disable the strain graph"]
#[usage = "[setting] [value]\nsettings [user]\nsettings copy [user]"]
async fn settings(ctx: &SerenityContext, msg: &Message) -> CommandResult {
    let author = if msg.mentions.is_empty() {
        msg.author.id
    } else {
        msg.mentions[0].id
    };

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

    if msg.content.split(' ').count() != 1
        && msg.content.split(' ').collect::<Vec<&str>>()[1] == "copy"
    {
        if msg.mentions.is_empty() {
            let content = "You need to mention someone in order to steal their settings!";
            msg.reply(&ctx, content).await?;
            return Ok(());
        }

        let copy_from = msg.mentions[0].id;
        if let Err(why) =
            tokio::fs::remove_file(format!("../danser/settings/{}.json", msg.author.id)).await
        {
            msg.reply(
                &ctx,
                "Oops! I couldn't find your file! Please type !!settings to resolve this issue.",
            )
            .await?;
            info!("User {} had error: {}", msg.author.name, why);
        }

        tokio::fs::copy(
            format!("../danser/settings/{}.json", copy_from),
            format!("../danser/settings/{}.json", msg.author.id),
        )
        .await?;

        msg.reply(&ctx, "Copied settings!").await?;
        return Ok(());
    }

    if msg.content.split(' ').count() == 3 {
        let new_settings = msg.content.split(' ').collect::<Vec<&str>>();

        match edit_setting(&mut settings, new_settings[1], new_settings[2], msg).await {
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

    let author_name = author.to_user(&ctx).await?.name;

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
                e.title(format!("Settings for {}", author_name))
                    .description(format!(
                        "**Skin**\n`skin`: {}\n\n\
                        **Cursor**\n`cursor size`: {}\n`cursor ripple`: {}\n\n\
                        **Beatmap**\n`storyboard`: {}\n`background video`: {}\n`dim`: {}\n\n\
                        **Audio**\n`music volume`: {}%\n`hitsound volume`: {}%\n`beatmap hitsounds`: {}\n\n\
                        **PP Counter**\n`pp counter decimals`: {}\n\n\
                        **Hit Error Meter**\n`hit error decimals`: {}\n\n\
                        **Aim Error Meter**\n`show aim error meter`: {}\n`aim error meter ur decimals`: {}\n\n\
                        **Hit Counter**\n`show hit counter`: {}\n`show sliderbreaks`: {}\n\n\
                        **Strain Graph**\n`show strain graph`: {}",
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
                        if !settings.audio.ignore_beatmap_samples {
                            "on"
                        } else {
                            "off"
                        },
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
                        },
                        if settings.gameplay.strain_graph.show {
                            "on"
                        } else {
                            "off"
                        }
                    ))
                    .color(Color::new(15785176))
                    .footer(|f| f.text("To edit your settings type !!settings [setting] [value] | The setting name is the same as in the embed, spaces are replaced with '_'"))
            })
        })
        .await?;
    Ok(())
}

async fn path_exists(path: String) -> bool {
    fs::metadata(path).await.is_ok()
}

#[derive(Debug, thiserror::Error)]
enum EditSettingsError {
    #[error("Aim error meter ur decimals have to be between 0 and 3!")]
    InvalidAimErrorDecimals,
    #[error("Cursorsize has to be between 0.1 and 2!")]
    InvalidCursorSize,
    #[error("Dim has to be between 0.0 and 1.0!")]
    InvalidDim,
    #[error("Hit error decimals have to be between 0 and 3!")]
    InvalidHitErrorDecimals,
    #[error("Hitsound volume has to be between 0 and 100!")]
    InvalidHitsoundVolume,
    #[error("Music volume has to be between 0 and 100!")]
    InvalidMusicVolume,
    #[error("PP counter decimals have to be between 0 and 3!")]
    InvalidPpCounterDecimals,
    #[error("Skin is not valid!")]
    InvalidSkin,
    #[error("Value is not valid!")]
    InvalidValue,
    #[error("Couldn't find skin!")]
    MissingSkin,
    #[error("The setting you tried to edit doesn't exist!")]
    InvalidSetting,
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

            if value_as_number < 0.0 || value_as_number > 100.0 {
                return Err(EditSettingsError::InvalidMusicVolume);
            }

            settings.audio.music_volume = value_as_number / 100.0;
        }
        "hitsound_volume" | "hitsound" => {
            let value_as_number: f64 = value
                .trim_end_matches('%')
                .parse()
                .map_err(|_| EditSettingsError::InvalidValue)?;

            if value_as_number < 0.0 || value_as_number > 100.0 {
                return Err(EditSettingsError::InvalidHitsoundVolume);
            }

            settings.audio.sample_volume = value_as_number / 100.0;
        }
        "beatmap_hitsounds" => {
            settings.audio.ignore_beatmap_samples =
                matches!(value.to_uppercase().as_str(), "OFF" | "FALSE" | "NO");
        }
        "pp_counter_decimals" => {
            let value_as_number: u64 =
                value.parse().map_err(|_| EditSettingsError::InvalidValue)?;

            if value_as_number > 3 {
                return Err(EditSettingsError::InvalidPpCounterDecimals);
            }

            settings.gameplay.pp_counter.decimals = value_as_number;
        }
        "hit_error_decimals" => {
            let value_as_number: u64 =
                value.parse().map_err(|_| EditSettingsError::InvalidValue)?;

            if value_as_number > 3 {
                return Err(EditSettingsError::InvalidHitErrorDecimals);
            }

            settings.gameplay.hit_error_meter.unstable_rate_decimals = value_as_number;
        }
        "aim_error_meter_ur_decimals" => {
            let value_as_number: u64 =
                value.parse().map_err(|_| EditSettingsError::InvalidValue)?;

            if value_as_number > 3 {
                return Err(EditSettingsError::InvalidAimErrorDecimals);
            }

            settings.gameplay.aim_error_meter.unstable_rate_decimals = value_as_number;
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
            settings.gameplay.hit_counter.show_sliderbreaks =
                matches!(value.to_uppercase().as_str(), "ON" | "TRUE" | "YES");
        }
        "show_strain_graph" | "strain_graph" => {
            settings.gameplay.strain_graph.show =
                matches!(value.to_uppercase().as_str(), "ON" | "TRUE" | "YES");
        }
        _ => {
            return Err(EditSettingsError::InvalidSetting);
        }
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
