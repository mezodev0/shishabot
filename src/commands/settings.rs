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
        edit_setting(
            &mut settings,
            &new_settings[1],
            &new_settings[2],
            &ctx,
            &msg,
        )
        .await;
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
                        **Gameplay**\n`pp counter decimals`: {}\n`hit error decimals`: {}\n\
                        `aim error meter`: {}\n`aim error meter ur decimals`: {}",
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

async fn edit_setting(
    mut settings: &mut Settings,
    key: &str,
    value: &str,
    ctx: &SerenityContext,
    msg: &Message,
) {
    if key == "skin" {
        if value.parse::<i32>().is_err() {
            if let Err(why) = msg.reply(&ctx, "Skin is not valid!").await {
                warn!("Couldn't send message: {}", why);
            }
            return;
        }

        let mut skins = fs::read_dir("../Skins/").await.unwrap();
        let mut counter = 0;
        let value_as_number = value.parse::<i32>().unwrap();
        let mut skin_found = false;

        while let Some(skin) = skins.next_entry().await.unwrap() {
            let file_name = skin.file_name();
            counter += 1;
            if counter == value_as_number {
                settings.skin.current_skin = file_name.into_string().unwrap();
                skin_found = true;
                break;
            }
        }

        if !skin_found {
            if let Err(why) = msg.reply(&ctx, "Couldn't find skin!").await {
                warn!("Couldn't send message: {}", why);
            }
            return;
        }
    } else if key == "cursor_size" {
        if value.parse::<f64>().is_err() {
            if let Err(why) = msg.reply(&ctx, "Value is not valid!").await {
                warn!("Couldn't send message: {}", why);
            }
            return;
        }

        let value_as_number = value.parse::<f64>().unwrap();
        if value_as_number < 0.1 || value_as_number > 2.0 {
            if let Err(why) = msg
                .reply(&ctx, "Cursorsize has to be between 0.1 and 2!")
                .await
            {
                warn!("Couldn't send message: {}", why);
            }
            return;
        }
        settings.skin.cursor.scale = value_as_number;
    } else if key == "cursor_ripple" {
        let value_capitalized = value.to_uppercase();
        if value_capitalized == "ON" || value_capitalized == "TRUE" || value_capitalized == "YES" {
            settings.cursor.cursor_ripples = true;
        } else {
            settings.cursor.cursor_ripples = false;
        }
    } else if key == "storyboard" {
        let value_capitalized = value.to_uppercase();
        if value_capitalized == "ON" || value_capitalized == "TRUE" || value_capitalized == "YES" {
            settings.playfield.background.load_storyboards = true;
        } else {
            settings.playfield.background.load_storyboards = false;
        }
    } else if key == "background_video" || key == "video" {
        let value_capitalized = value.to_uppercase();
        if value_capitalized == "ON" || value_capitalized == "TRUE" || value_capitalized == "YES" {
            settings.playfield.background.load_videos = true;
        } else {
            settings.playfield.background.load_videos = false;
        }
    } else if key == "dim" {
        if value.parse::<f64>().is_err() {
            if let Err(why) = msg.reply(&ctx, "Value is not valid!").await {
                warn!("Couldn't send message: {}", why);
            }
            return;
        }

        let value_as_number = value.parse::<f64>().unwrap();
        if value_as_number < 0.0 || value_as_number > 1.0 {
            if let Err(why) = msg.reply(&ctx, "Dim has to be between 0 and 1!").await {
                warn!("Couldn't send message: {}", why);
            }
            return;
        }
        settings.playfield.background.dim.normal = value_as_number;
    } else if key == "music_volume" || key == "music" {
        let clean_value = value.replace("%", "");
        if clean_value.parse::<u64>().is_err() {
            if let Err(why) = msg.reply(&ctx, "Value is not valid!").await {
                warn!("Couldn't send message: {}", why);
            }
            return;
        }
        let value_as_number = value.parse::<u64>().unwrap();
        if value_as_number < 1 || value_as_number > 100 {
            if let Err(why) = msg
                .reply(&ctx, "Music volume has to be between 1 and 100!")
                .await
            {
                warn!("Couldn't send message: {}", why);
            }
            return;
        }
        let value_as_float: f64 = (value_as_number / 100) as f64;
        settings.audio.music_volume = value_as_float;
    } else if key == "hitsound_volume" || key == "hitsound" {
        let clean_value = value.replace("%", "");
        if clean_value.parse::<u64>().is_err() {
            if let Err(why) = msg.reply(&ctx, "Value is not valid!").await {
                warn!("Couldn't send message: {}", why);
            }
            return;
        }
        let value_as_number = value.parse::<u64>().unwrap();
        if value_as_number < 1 || value_as_number > 100 {
            if let Err(why) = msg
                .reply(&ctx, "Hitsound volume has to be between 1 and 100!")
                .await
            {
                warn!("Couldn't send message: {}", why);
            }
            return;
        }
        let value_as_float: f64 = (value_as_number / 100) as f64;
        settings.audio.sample_volume = value_as_float;
    } else if key == "pp_counter_decimals" {
        if value.parse::<u64>().is_err() {
            if let Err(why) = msg.reply(&ctx, "Value is not valid!").await {
                warn!("Couldn't send message: {}", why);
            }
            return;
        }
        let value_as_number = value.parse::<u64>().unwrap();
        if value_as_number < 1 || value_as_number > 3 {
            if let Err(why) = msg
                .reply(&ctx, "PP counter decimals have to be between 1 and 3!")
                .await
            {
                warn!("Couldn't send message: {}", why);
            }
            return;
        }
        settings.gameplay.pp_counter.decimals = value_as_number;
    } else if key == "hit_error_decimals" {
        if value.parse::<u64>().is_err() {
            if let Err(why) = msg.reply(&ctx, "Value is not valid!").await {
                warn!("Couldn't send message: {}", why);
            }
            return;
        }
        let value_as_number = value.parse::<u64>().unwrap();
        if value_as_number < 1 || value_as_number > 3 {
            if let Err(why) = msg
                .reply(&ctx, "Hit error decimals have to be between 1 and 3!")
                .await
            {
                warn!("Couldn't send message: {}", why);
            }
            return;
        }
        settings.gameplay.pp_counter.decimals = value_as_number;
    } else if key == "aim_error_meter_ur_decimals" {
        if value.parse::<u64>().is_err() {
            if let Err(why) = msg.reply(&ctx, "Value is not valid!").await {
                warn!("Couldn't send message: {}", why);
            }
            return;
        }
        let value_as_number = value.parse::<u64>().unwrap();
        if value_as_number < 1 || value_as_number > 3 {
            if let Err(why) = msg
                .reply(
                    &ctx,
                    "Aim error meter ur decimals have to be between 1 and 3!",
                )
                .await
            {
                warn!("Couldn't send message: {}", why);
            }
            return;
        }
        settings.gameplay.pp_counter.decimals = value_as_number;
    } else if key == "aim_error_meter" {
        let value_capitalized = value.to_uppercase();
        if value_capitalized == "ON" || value_capitalized == "TRUE" || value_capitalized == "YES" {
            settings.gameplay.aim_error_meter.show = true;
        } else {
            settings.gameplay.aim_error_meter.show = false;
        }
    }

    let edited_setting = serde_json::to_string(&settings).unwrap();
    if let Err(why) = tokio::fs::write(
        format!("../danser/settings/{}.json", msg.author.id),
        edited_setting,
    )
    .await
    {
        let err = Error::new(why).context(format!(
            "failed writing to `../danser/settings/{}.json` on edit_setting",
            msg.author.id
        ));
        warn!("{:?}", err);
    }
    msg.reply(&ctx, "Edited setting successfully!")
        .await
        .unwrap();
}
