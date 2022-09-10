use std::{
    convert::TryInto,
    fs::{File, OpenOptions},
    path::PathBuf,
    sync::Arc,
};

use eyre::{Context as _, ContextCompat, Report, Result};
use twilight_model::application::command::CommandOptionChoice;

use crate::{
    core::{settings::DanserSettings, BotConfig, Context},
    util::{
        builder::MessageBuilder, interaction::InteractionCommand, levenshtein_distance, Authored,
        CowUtils, InteractionCommandExt,
    },
};

use super::{create_settings_embed, SettingsEdit, SettingsEditAutocomplete, Visibility};

pub async fn edit(
    ctx: Arc<Context>,
    command: InteractionCommand,
    args: SettingsEditAutocomplete,
) -> Result<()> {
    let args: SettingsEdit = match args.try_into() {
        Ok(args) => args,
        Err(autocomplete) => {
            if autocomplete.is_empty() {
                command.autocomplete(&ctx, Vec::new()).await?;

                return Ok(());
            }

            let no_underscores = autocomplete.cow_replace('_', " ");
            let skin = no_underscores.cow_to_ascii_lowercase();

            let mut skins: Vec<_> = ctx
                .skin_list()
                .get()?
                .iter()
                .map(|skin| {
                    skin.to_string_lossy()
                        .cow_replace('_', " ")
                        .to_ascii_lowercase()
                })
                .filter(|haystack| haystack.contains(skin.as_ref()))
                .take(25)
                .collect();

            skins.sort_unstable_by_key(|opt| levenshtein_distance(opt, skin.as_ref()).0);

            let choices = skins
                .into_iter()
                .map(|skin| CommandOptionChoice::String {
                    name: skin.clone(),
                    name_localizations: None,
                    value: skin,
                })
                .collect();

            command.autocomplete(&ctx, choices).await?;

            return Ok(());
        }
    };

    let author = command.user_id()?;
    let danser_path = BotConfig::get().paths.danser();

    let mut user_path = danser_path.to_owned();
    user_path.push(format!("settings/{author}.json"));

    // Retrieve attachment if provided
    let attached = if let Some(ref attachment) = args.file {
        if !matches!(attachment.filename.rsplit('.').next(), Some("json")) {
            let content = "The attached file must be of type .json";
            command.error_callback(&ctx, content, false).await?;

            return Ok(());
        }

        match ctx.client().get_discord_attachment(attachment).await {
            Ok(bytes) => match serde_json::from_slice::<DanserSettings>(&bytes) {
                Ok(settings) => Some(settings),
                Err(err) => {
                    let content = format!(
                        "Failed to deserialize the attached .json data into danser settings: {err}\n\
                        Be sure you provide valid danser settings."
                    );
                    command.error_callback(&ctx, content, false).await?;

                    return Ok(());
                }
            },
            Err(err) => {
                let content = "Failed to download the provided attachment";
                let _ = command.error_callback(&ctx, content, false).await;

                return Err(err.wrap_err("failed to download settings attachment"));
            }
        }
    } else {
        None
    };

    let mut settings: DanserSettings = if user_path.exists() {
        let file = File::open(&user_path)
            .with_context(|| format!("failed to open settings file at {user_path:?}"))?;

        serde_json::from_reader(file)
            .with_context(|| format!("failed to deserialize settings file at {user_path:?}"))?
    } else {
        let mut default_path = danser_path.to_owned();
        default_path.push("settings/default.json");
        let file = File::open(default_path).context("failed to open default settings file")?;

        serde_json::from_reader(file).context("failed to deserialize default settings file")?
    };

    if let Some(attached) = attached {
        settings.audio = attached.audio;
        settings.input = attached.input;
        settings.gameplay = attached.gameplay;
        settings.cursor = attached.cursor;
        settings.objects = attached.objects;
        settings.playfield = attached.playfield;

        settings.skin.use_colors_from_skin = attached.skin.use_colors_from_skin;
        settings.skin.use_beatmap_colors = attached.skin.use_beatmap_colors;
        settings.skin.cursor = attached.skin.cursor;
    }

    // Modify the settings and check if something changed
    match modify_settings(&ctx, &mut settings, args) {
        ModifyResult::Change(true) => {
            let file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&user_path)
                .with_context(|| {
                    format!("failed to open file at {user_path:?} after modifying settings")
                })?;

            match serde_json::to_writer(file, &settings) {
                Ok(_) => {
                    let user = command.user()?;
                    let embed = create_settings_embed(user, &settings);
                    let content = "Successfully changed settings!";
                    let builder = MessageBuilder::new().content(content).embed(embed);
                    command.callback(&ctx, builder, false).await?;

                    Ok(())
                }
                Err(err) => {
                    let content = "Failed to save modified settings";
                    let _ = command.error_callback(&ctx, content, false).await;

                    let err = Report::from(err)
                        .wrap_err(format!("failed to serialize settings into {user_path:?}"));

                    Err(err)
                }
            }
        }
        ModifyResult::Change(false) => {
            let user = command.user()?;
            let embed = create_settings_embed(user, &settings);
            let builder = MessageBuilder::new().embed(embed);
            command.callback(&ctx, builder, false).await?;

            Ok(())
        }
        ModifyResult::SkinNotFound => {
            let content = "No skin with the specified name is stored.\n\
                Check `/skinlist` to see available skins.";
            command.error_callback(&ctx, content, false).await?;

            Ok(())
        }
        ModifyResult::Err(err) => {
            let content = "Something went wrong while modifying the settings";
            let _ = command.error_callback(&ctx, content, false).await;

            Err(err.wrap_err("failed to modify settings"))
        }
    }
}

enum ModifyResult {
    Change(bool),
    SkinNotFound,
    Err(Report),
}

fn modify_settings(
    ctx: &Context,
    settings: &mut DanserSettings,
    args: SettingsEdit,
) -> ModifyResult {
    let SettingsEdit {
        file: _,
        skin,
        cursor_scale,
        cursor_ripples,
        storyboard,
        video,
        dim,
        music_volume,
        hitsound_volume,
        leaderboard,
        beatmap_hitsounds,
        pp,
        pp_decimals,
        hit_error_meter,
        hit_error_decimals,
        aim_error_meter,
        aim_error_decimals,
        hit_counter,
        sliderbreaks,
        strain_graph,
    } = args;

    let mut changed = false;

    if let Some(skin) = skin {
        let skin_name_res = ctx.skin_list().get().map(|skins| {
            let skin = skin.cow_replace('_', " ");

            skins.iter().find_map(|skin_| {
                let s = skin_.to_ascii_lowercase();
                let s = s.to_string_lossy();
                let s = s.cow_replace('_', " ");

                (s == skin).then(|| skin_.to_owned())
            })
        });

        let skin_name = match skin_name_res {
            Ok(Some(name)) => name,
            Ok(None) => return ModifyResult::SkinNotFound,
            Err(err) => return ModifyResult::Err(err),
        };

        // Get skin of current settings
        let mut curr_skin_path = PathBuf::from(&settings.skin.current_skin);

        let curr_skin_res = curr_skin_path
            .file_name()
            .with_context(|| format!("missing filename for skin path {curr_skin_path:?}"));

        let is_different_skin = match curr_skin_res {
            Ok(skin) => skin != skin_name,
            Err(err) => return ModifyResult::Err(err),
        };

        // Compare new with old
        if is_different_skin {
            curr_skin_path.pop();
            curr_skin_path.push(skin_name);
            settings.skin.current_skin = curr_skin_path.to_string_lossy().into_owned();
            changed = true;
        }
    }

    // Some simple macros:
    // - Compare $new with $field
    // - If they're different, assign $new to $field and set $changed to true

    macro_rules! assign_f64 {
        ($changed:ident: $($new:ident ~ $field:expr;)*) => {
            $(
                if let Some($new) = $new {
                    if ($field - $new).abs() > f64::EPSILON {
                        $field = $new;
                        $changed = true;
                    }
                }
            )*
        };
    }

    macro_rules! assign_cmp {
        ($changed:ident: $($new:ident ~ $field:expr;)*) => {
            $(
                if let Some($new) = $new {
                    if $field != $new {
                        $field = $new;
                        $changed = true;
                    }
                }
            )*
        };
    }

    macro_rules! assign_reverse_cmp {
        ($changed:ident: $($new:ident ~ $field:expr;)*) => {
            $(
                if let Some($new) = $new {
                    if $field != !$new {
                        $field = !$new;
                        $changed = true;
                    }
                }
            )*
        };
    }

    macro_rules! assign_visibility {
        ($changed:ident: $($new:ident ~ $field:expr;)*) => {
            $(
                if let Some($new) = $new.map(|vis| matches!(vis, Visibility::Show)) {
                    if $field != $new {
                        $field = $new;
                        $changed = true;
                    }
                }
            )*
        };
    }

    macro_rules! assign_percent {
        ($changed:ident: $($new:ident ~ $field:expr;)*) => {
            $(
                if let Some($new) = $new {
                    let mapped = $new as f64 / 100.0;

                    if  ($field - mapped).abs() > f64::EPSILON {
                        $field = mapped;
                        $changed = true;
                    }
                }
            )*
        };
    }

    assign_f64! { changed:
        cursor_scale ~ settings.skin.cursor.scale;
    };

    assign_cmp! { changed:
        cursor_ripples ~ settings.cursor.cursor_ripples;
        leaderboard ~ settings.gameplay.score_board.show;
        storyboard ~ settings.playfield.background.load_storyboards;
        video ~ settings.playfield.background.load_videos;
        pp_decimals ~ settings.gameplay.pp_counter.decimals;
        hit_error_decimals ~ settings.gameplay.hit_error_meter.unstable_rate_decimals;
        aim_error_decimals ~ settings.gameplay.aim_error_meter.unstable_rate_decimals;
    }

    assign_reverse_cmp! { changed:
        beatmap_hitsounds ~ settings.audio.ignore_beatmap_samples;
    }

    assign_visibility! { changed:
        pp ~ settings.gameplay.pp_counter.show;
        hit_error_meter ~ settings.gameplay.hit_error_meter.show;
        aim_error_meter ~ settings.gameplay.aim_error_meter.show;
        hit_counter ~ settings.gameplay.hit_counter.show;
        sliderbreaks ~ settings.gameplay.hit_counter.show_sliderbreaks;
        strain_graph ~ settings.gameplay.strain_graph.show;
    }

    assign_percent! { changed:
        dim ~ settings.playfield.background.dim.normal;
        music_volume ~ settings.audio.music_volume;
        hitsound_volume ~ settings.audio.sample_volume;
    }

    ModifyResult::Change(changed)
}
