use std::{
    convert::TryFrom,
    fmt::{Display, Formatter, Result as FmtResult},
    path::PathBuf,
    sync::Arc,
};

use command_macros::SlashCommand;
use eyre::Result;
use twilight_interactions::command::{
    AutocompleteValue, CommandModel, CommandOption, CreateCommand, CreateOption, ResolvedUser,
};
use twilight_model::{
    channel::{
        embed::{Embed, EmbedField},
        Attachment,
    },
    id::{marker::UserMarker, Id},
    user::User,
};

use crate::{
    core::{settings::DanserSettings, Context},
    util::{builder::EmbedBuilder, interaction::InteractionCommand, InteractionCommandExt},
};

use self::{copy::*, default::*, edit::*, view::*};

mod copy;
mod default;
mod edit;
mod view;

#[derive(CreateCommand, SlashCommand)]
#[command(name = "settings")]
#[flags(SKIP_DEFER)]
#[allow(unused)]
/// Adjust your danser settings
pub enum Settings {
    #[command(name = "copy")]
    Copy(SettingsCopy),
    #[command(name = "default")]
    Default(SettingsDefault),
    #[command(name = "edit")]
    Edit(SettingsEdit),
    #[command(name = "view")]
    View(SettingsView),
}

#[derive(CommandModel)]
pub enum SettingsParsable {
    #[command(name = "copy")]
    Copy(SettingsCopy),
    #[command(name = "default")]
    Default(SettingsDefault),
    #[command(name = "edit")]
    Edit(SettingsEditAutocomplete),
    #[command(name = "view")]
    View(SettingsView),
}

#[derive(CreateCommand, CommandModel)]
#[command(name = "copy")]
/// Copy over the settings from another user
pub struct SettingsCopy {
    /// User from which the settings will be copied
    user: Id<UserMarker>,
}

#[derive(CreateCommand, CommandModel)]
#[command(name = "default")]
/// Restore your settings to the default values
pub struct SettingsDefault {
    /// Confirm that you want to overwrite your current settings
    confirm: SettingsConfirm,
}

#[derive(CreateOption, CommandOption)]
pub enum SettingsConfirm {
    #[option(name = "Oops missclick", value = "cancel")]
    Cancel,
    #[option(name = "I want to overwrite my current settings", value = "confirm")]
    Confirm,
}

#[derive(CreateCommand)]
#[command(name = "edit")]
/// Make specific changes to your settings
pub struct SettingsEdit {
    /// Provide your own .json settings file
    file: Option<Attachment>,
    #[command(autocomplete = true)]
    /// Specify one of the available skins
    skin: Option<String>,
    #[command(min_value = 0.1, max_value = 2.0)]
    /// Scale the size of the cursor
    cursor_scale: Option<f64>,
    /// Whether the cursor should generate ripples
    cursor_ripples: Option<bool>,
    /// Whether the leaderboard should be displayed
    leaderboard: Option<bool>,
    /// Whether the storyboard should play
    storyboard: Option<bool>,
    /// Whether the video should play
    video: Option<bool>,
    #[command(min_value = 0, max_value = 100)]
    /// Percent background dim
    dim: Option<u8>,
    #[command(min_value = 0, max_value = 100)]
    /// Percent music volume
    music_volume: Option<u8>,
    #[command(min_value = 0, max_value = 100)]
    /// Percent hitsound volume
    hitsound_volume: Option<u8>,
    /// Whether the beatmap's hitsounds should be used
    beatmap_hitsounds: Option<bool>,
    /// Whether pp should be displayed
    pp: Option<Visibility>,
    #[command(min_value = 0, max_value = 3)]
    /// How many decimal places the pp value should have
    pp_decimals: Option<u32>,
    /// Whether the hit error meter should be displayed
    hit_error_meter: Option<Visibility>,
    #[command(min_value = 0, max_value = 3)]
    /// How many decimal places the hit error should have
    hit_error_decimals: Option<u32>,
    /// Whether the aim error meter should be displayed
    aim_error_meter: Option<Visibility>,
    #[command(min_value = 0, max_value = 3)]
    /// How many decimal place sthe aim error UR should have
    aim_error_decimals: Option<u32>,
    /// Whether the hit counter should be displayed
    hit_counter: Option<Visibility>,
    /// Whether sliderbreaks should be displayed
    sliderbreaks: Option<Visibility>,
    /// Whether the strain graph should be displayed
    strain_graph: Option<Visibility>,
}

impl TryFrom<SettingsEditAutocomplete> for SettingsEdit {
    type Error = String;

    #[inline]
    fn try_from(edit: SettingsEditAutocomplete) -> Result<Self, Self::Error> {
        let SettingsEditAutocomplete {
            file,
            skin,
            cursor_scale,
            cursor_ripples,
            leaderboard,
            storyboard,
            video,
            dim,
            music_volume,
            hitsound_volume,
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
        } = edit;

        let skin = match skin {
            AutocompleteValue::Focused(skin) => return Err(skin),
            AutocompleteValue::None => None,
            AutocompleteValue::Completed(skin) => Some(skin),
        };

        let edit = Self {
            file,
            skin,
            cursor_scale,
            cursor_ripples,
            leaderboard,
            storyboard,
            video,
            dim,
            music_volume,
            hitsound_volume,
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
        };

        Ok(edit)
    }
}

#[derive(CommandModel)]
#[command(autocomplete = true)]
pub struct SettingsEditAutocomplete {
    file: Option<Attachment>,
    skin: AutocompleteValue<String>,
    cursor_scale: Option<f64>,
    cursor_ripples: Option<bool>,
    leaderboard: Option<bool>,
    storyboard: Option<bool>,
    video: Option<bool>,
    dim: Option<u8>,
    music_volume: Option<u8>,
    hitsound_volume: Option<u8>,
    beatmap_hitsounds: Option<bool>,
    pp: Option<Visibility>,
    pp_decimals: Option<u32>,
    hit_error_meter: Option<Visibility>,
    hit_error_decimals: Option<u32>,
    aim_error_meter: Option<Visibility>,
    aim_error_decimals: Option<u32>,
    hit_counter: Option<Visibility>,
    sliderbreaks: Option<Visibility>,
    strain_graph: Option<Visibility>,
}

#[derive(CreateOption, CommandOption)]
pub enum Visibility {
    #[option(name = "Show", value = "show")]
    Show,
    #[option(name = "Hide", value = "hide")]
    Hide,
}

#[derive(CreateCommand, CommandModel)]
#[command(name = "view")]
/// View the settings of a user
pub struct SettingsView {
    /// The user to view the settings of
    user: ResolvedUser,
}

pub async fn slash_settings(ctx: Arc<Context>, mut command: InteractionCommand) -> Result<()> {
    match SettingsParsable::from_interaction(command.input_data())? {
        SettingsParsable::Copy(args) => copy(ctx, command, args).await,
        SettingsParsable::Default(args) => default(ctx, command, args).await,
        SettingsParsable::Edit(args) => edit(ctx, command, args).await,
        SettingsParsable::View(args) => view(ctx, command, args).await,
    }
}

fn create_settings_embed(user: &User, settings: &DanserSettings) -> Embed {
    let skin_path = PathBuf::from(&settings.skin.current_skin);
    let skin = skin_path
        .file_name()
        .expect("missing skin file name")
        .to_string_lossy();

    let on_off = |b| if b { "on" } else { "off" };
    let percent = |value| Percent { value };

    let fields = vec![
        EmbedField {
            inline: false,
            name: "Skin".to_owned(),
            value: format!("`skin`: {skin}"),
        },
        EmbedField {
            inline: true,
            name: "Beatmap".to_owned(),
            value: format!(
                "`storyboard`: {}\n\
                `video`: {}\n\
                `dim`: {}\n\
                `leaderboard`: {}",
                on_off(settings.playfield.background.load_storyboards),
                on_off(settings.playfield.background.load_videos),
                percent(settings.playfield.background.dim.normal),
                on_off(settings.gameplay.score_board.show),
            ),
        },
        EmbedField {
            inline: true,
            name: "Audio".to_owned(),
            value: format!(
                "`music volume`: {}\n\
                `hitsound volume`: {}\n\
                `beatmap hitsounds`: {}",
                percent(settings.audio.music_volume),
                percent(settings.audio.sample_volume),
                on_off(!settings.audio.ignore_beatmap_samples),
            ),
        },
        EmbedField {
            inline: false,
            name: "Cursor".to_owned(),
            value: format!(
                "`cursor scale`: {}\n\
                `cursor ripples`: {}",
                settings.skin.cursor.scale,
                on_off(settings.cursor.cursor_ripples),
            ),
        },
        EmbedField {
            inline: true,
            name: "PP Counter".to_owned(),
            value: format!(
                "`show pp counter`: {}\n\
                `pp decimals`: {}",
                on_off(settings.gameplay.pp_counter.show),
                settings.gameplay.pp_counter.decimals,
            ),
        },
        EmbedField {
            inline: true,
            name: "Hit Counter".to_owned(),
            value: format!(
                "`show hit counter`: {}\n\
                `show sliderbreaks`: {}",
                on_off(settings.gameplay.hit_counter.show),
                on_off(settings.gameplay.hit_counter.show_sliderbreaks),
            ),
        },
        EmbedField {
            inline: false,
            name: "Strain Graph".to_owned(),
            value: format!(
                "`show strain graph`: {}",
                on_off(settings.gameplay.strain_graph.show),
            ),
        },
        EmbedField {
            inline: true,
            name: "Hit Error Meter".to_owned(),
            value: format!(
                "`show hit error meter`: {}\n\
                `hit error decimals`: {}",
                on_off(settings.gameplay.hit_error_meter.show),
                settings.gameplay.hit_error_meter.unstable_rate_decimals,
            ),
        },
        EmbedField {
            inline: true,
            name: "Aim Error Meter".to_owned(),
            value: format!(
                "`show aim error meter`: {}\n\
                `aim error UR decimals`: {}",
                on_off(settings.gameplay.aim_error_meter.show),
                settings.gameplay.aim_error_meter.unstable_rate_decimals,
            ),
        },
    ];

    EmbedBuilder::new()
        .title(format!("Settings for {}", user.name))
        .fields(fields)
        .build()
}

struct Percent {
    value: f64,
}

impl Display for Percent {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}%", (self.value * 100.0).round())
    }
}
