use std::collections::HashSet;

use flurry::HashMap as FlurryMap;
use serde::{Deserialize, Serialize};
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker},
    Id,
};

use crate::util::hasher::IntBuildHasher;

type Servers = FlurryMap<Id<GuildMarker>, Server, IntBuildHasher>;

#[derive(Debug, Deserialize, Serialize)]
pub struct RootSettings {
    #[serde(rename = "Servers", with = "servers")]
    pub servers: Servers,
}

#[derive(Clone, Debug, Default)]
pub struct Server {
    pub input_channels: HashSet<Id<ChannelMarker>, IntBuildHasher>,
    pub output_channel: Option<Id<ChannelMarker>>,
}

mod servers {
    use std::{
        collections::HashSet,
        fmt::{Formatter, Result as FmtResult},
    };

    use serde::{
        de::{SeqAccess, Visitor},
        ser::{SerializeSeq, SerializeStruct},
        Deserialize, Deserializer, Serialize, Serializer,
    };
    use twilight_model::id::{
        marker::{ChannelMarker, GuildMarker},
        Id,
    };

    use crate::util::hasher::IntBuildHasher;

    use super::{FlurryMap, Server, Servers};

    #[derive(Deserialize)]
    struct RawServer {
        server_id: Id<GuildMarker>,
        input_channels: HashSet<Id<ChannelMarker>, IntBuildHasher>,
        output_channel: Option<Id<ChannelMarker>>,
    }

    struct ServersVisitor;

    impl<'de> Visitor<'de> for ServersVisitor {
        type Value = Servers;

        fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
            f.write_str("a list of servers")
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let servers =
                FlurryMap::with_capacity_and_hasher(seq.size_hint().unwrap_or(0), IntBuildHasher);

            {
                let guard = servers.pin();

                while let Some(raw) = seq.next_element()? {
                    let RawServer {
                        server_id,
                        input_channels,
                        output_channel,
                    } = raw;

                    let server = Server {
                        input_channels,
                        output_channel,
                    };

                    guard.insert(server_id, server);
                }
            }

            Ok(servers)
        }
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Servers, D::Error> {
        d.deserialize_seq(ServersVisitor)
    }

    struct BorrowedRawServer<'s> {
        server_id: Id<GuildMarker>,
        server: &'s Server,
    }

    impl Serialize for BorrowedRawServer<'_> {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            let mut raw = s.serialize_struct("RawServer", 3)?;

            raw.serialize_field("server_id", &self.server_id)?;
            raw.serialize_field("input_channels", &self.server.input_channels)?;
            raw.serialize_field("output_channel", &self.server.output_channel)?;

            raw.end()
        }
    }

    pub(super) fn serialize<S: Serializer>(servers: &Servers, s: S) -> Result<S::Ok, S::Error> {
        let mut seq = s.serialize_seq(Some(servers.len()))?;

        for (&server_id, server) in servers.pin().iter() {
            let server = BorrowedRawServer { server_id, server };
            seq.serialize_element(&server)?;
        }

        seq.end()
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct DanserSettings {
    pub general: General,
    pub graphics: Graphics,
    pub audio: Audio,
    pub input: Input,
    pub gameplay: Gameplay,
    pub skin: Skin,
    pub cursor: Cursor,
    pub objects: Objects,
    pub playfield: Playfield,
    pub cursor_dance: CursorDance,
    pub knockout: Knockout,
    pub recording: Recording,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct General {
    pub osu_songs_dir: String,
    pub osu_skins_dir: String,
    pub osu_replays_dir: String,
    pub discord_presence_on: bool,
    pub unpack_osz_files: bool,
    pub verbose_import_logs: bool,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Graphics {
    pub width: u32,
    pub height: u32,
    pub window_width: u32,
    pub window_height: u32,
    pub fullscreen: bool,
    #[serde(rename = "VSync")]
    pub vsync: bool,
    #[serde(rename = "FPSCap")]
    pub fpscap: u32,
    #[serde(rename = "MSAA")]
    pub msaa: i8,
    #[serde(rename = "ShowFPS")]
    pub show_fps: bool,
    pub experimental: Experimental,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Experimental {
    pub use_persistent_buffers: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Audio {
    pub general_volume: f64,
    pub music_volume: f64,
    pub sample_volume: f64,
    pub offset: i32,
    pub hitsound_position_multiplier: f64,
    pub ignore_beatmap_samples: bool,
    pub ignore_beatmap_sample_volume: bool,
    pub play_nightcore_samples: bool,
    pub beat_scale: f64,
    pub beat_use_timing_points: bool,
    #[serde(rename = "Linux/Unix")]
    pub linux_unix: LinuxUnix,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct LinuxUnix {
    pub bass_playback_buffer_length: i32,
    pub bass_device_buffer_length: i32,
    pub bass_update_period: i64,
    pub bass_device_update_period: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Input {
    pub left_key: String,
    pub right_key: String,
    pub restart_key: String,
    pub smoke_key: String,
    pub screenshot_key: String,
    pub mouse_buttons_disabled: bool,
    pub mouse_high_precision: bool,
    pub mouse_sensitivity: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Gameplay {
    pub hit_error_meter: HitErrorMeter,
    pub aim_error_meter: AimErrorMeter,
    pub score: Score,
    pub hp_bar: HpBar,
    pub combo_counter: ComboCounter,
    #[serde(rename = "PPCounter")]
    pub pp_counter: PpCounter,
    pub hit_counter: HitCounter,
    pub strain_graph: StrainGraph,
    pub key_overlay: KeyOverlay,
    pub score_board: ScoreBoard,
    pub mods: Mods,
    pub boundaries: Boundaries,
    pub underlay: Underlay,
    #[serde(rename = "HUDFont")]
    pub hud_font: String,
    pub show_results_screen: bool,
    pub results_screen_time: i32,
    pub results_use_local_time_zone: bool,
    pub show_warning_arrows: bool,
    pub show_hit_lighting: bool,
    pub flashlight_dim: f64,
    pub play_username: String,
    #[serde(rename = "UseLazerPP")]
    pub use_lazer_pp: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct HitErrorMeter {
    pub show: bool,
    pub scale: f64,
    pub opacity: f64,
    #[serde(rename = "XOffset")]
    pub xoffset: f64,
    #[serde(rename = "YOffset")]
    pub yoffset: f64,
    pub point_fade_out_time: f64,
    pub show_positional_misses: bool,
    pub positional_miss_scale: f64,
    pub show_unstable_rate: bool,
    pub unstable_rate_decimals: u32,
    pub unstable_rate_scale: f64,
    pub static_unstable_rate: bool,
    pub scale_with_speed: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct AimErrorMeter {
    pub show: bool,
    pub scale: f64,
    pub opacity: f64,
    #[serde(rename = "XPosition")]
    pub xposition: f64,
    #[serde(rename = "YPosition")]
    pub yposition: f64,
    pub point_fade_out_time: f64,
    pub dot_scale: f64,
    pub align: String,
    pub show_unstable_rate: bool,
    pub unstable_rate_scale: f64,
    pub unstable_rate_decimals: u32,
    pub static_unstable_rate: bool,
    pub cap_positional_misses: bool,
    pub angle_normalized: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Score {
    pub show: bool,
    pub scale: f64,
    pub opacity: f64,
    #[serde(rename = "XOffset")]
    pub xoffset: f64,
    #[serde(rename = "YOffset")]
    pub yoffset: f64,
    pub progress_bar: String,
    pub show_grade_always: bool,
    pub static_score: bool,
    pub static_accuracy: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct HpBar {
    pub show: bool,
    pub scale: f64,
    pub opacity: f64,
    #[serde(rename = "XOffset")]
    pub xoffset: f64,
    #[serde(rename = "YOffset")]
    pub yoffset: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct ComboCounter {
    pub show: bool,
    pub scale: f64,
    pub opacity: f64,
    #[serde(rename = "XOffset")]
    pub xoffset: f64,
    #[serde(rename = "YOffset")]
    pub yoffset: f64,
    #[serde(rename = "Static")]
    pub static_combo: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct PpCounter {
    pub show: bool,
    pub scale: f64,
    pub opacity: f64,
    #[serde(rename = "XPosition")]
    pub xposition: f64,
    #[serde(rename = "YPosition")]
    pub yposition: f64,
    pub color: Hsv,
    pub decimals: u32,
    pub align: String,
    pub show_in_results: bool,
    #[serde(rename = "ShowPPComponents")]
    pub show_pp_components: bool,
    #[serde(rename = "Static")]
    pub static_pp: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Hsv {
    pub hue: f64,
    pub saturation: f64,
    pub value: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct HitCounter {
    pub show: bool,
    pub scale: f64,
    pub opacity: f64,
    #[serde(rename = "XPosition")]
    pub xposition: f64,
    #[serde(rename = "YPosition")]
    pub yposition: f64,
    #[serde(rename = "Color300")]
    pub color_300: Hsv,
    #[serde(rename = "Color100")]
    pub color_100: Hsv,
    #[serde(rename = "Color50")]
    pub color_50: Hsv,
    #[serde(rename = "ColorMiss")]
    pub color_miss: Hsv,
    #[serde(rename = "ColorSB")]
    pub color_sb: Hsv,
    pub spacing: i64,
    pub font_scale: i64,
    pub align: String,
    pub value_align: String,
    pub vertical: bool,
    #[serde(rename = "Show300")]
    pub show300: bool,
    #[serde(rename = "ShowSliderBreaks")]
    pub show_sliderbreaks: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct StrainGraph {
    pub show: bool,
    pub opacity: f64,
    #[serde(rename = "XPosition")]
    pub xposition: f64,
    #[serde(rename = "YPosition")]
    pub yposition: f64,
    pub align: String,
    pub width: f64,
    pub height: f64,
    pub bg_color: Hsv,
    pub fg_color: Hsv,
    pub outline: StrainGraphOutline,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct StrainGraphOutline {
    pub show: bool,
    pub width: f64,
    pub inner_darkness: f64,
    pub inner_opacity: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct KeyOverlay {
    pub show: bool,
    pub scale: f64,
    pub opacity: f64,
    #[serde(rename = "XOffset")]
    pub xoffset: f64,
    #[serde(rename = "YOffset")]
    pub yoffset: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct ScoreBoard {
    pub show: bool,
    pub scale: f64,
    pub opacity: f64,
    #[serde(rename = "XOffset")]
    pub xoffset: f64,
    #[serde(rename = "YOffset")]
    pub yoffset: f64,
    pub mods_only: bool,
    pub align_right: bool,
    pub hide_others: bool,
    pub show_avatars: bool,
    pub explosion_scale: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Mods {
    pub show: bool,
    pub scale: f64,
    pub opacity: f64,
    #[serde(rename = "XOffset")]
    pub xoffset: f64,
    #[serde(rename = "YOffset")]
    pub yoffset: f64,
    pub hide_in_replays: bool,
    pub fold_in_replays: bool,
    pub additional_spacing: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Boundaries {
    pub enabled: bool,
    pub border_thickness: f64,
    pub border_fill: f64,
    pub border_color: Hsv,
    pub border_opacity: f64,
    pub background_color: Hsv,
    pub background_opacity: f64,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Underlay {
    pub path: String,
    pub above_hp_bar: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Skin {
    pub current_skin: String,
    pub fallback_skin: String,
    pub use_colors_from_skin: bool,
    pub use_beatmap_colors: bool,
    pub cursor: SkinCursor,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct SkinCursor {
    pub use_skin_cursor: bool,
    pub scale: f64,
    pub trail_scale: f64,
    pub force_long_trail: bool,
    pub long_trail_length: i32,
    pub long_trail_density: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Cursor {
    pub trail_style: u8,
    #[serde(rename = "Style23Speed")]
    pub style23speed: f64,
    #[serde(rename = "Style4Shift")]
    pub style4shift: f64,
    pub colors: CursorColors,
    pub enable_custom_tag_color_offset: bool,
    pub tag_color_offset: f64,
    pub enable_trail_glow: bool,
    pub enable_custom_trail_glow_offset: bool,
    pub trail_glow_offset: i32,
    #[serde(rename = "ScaleToCS")]
    pub scale_to_cs: bool,
    pub cursor_size: i32,
    pub cursor_expand: bool,
    pub scale_to_the_beat: bool,
    pub show_cursors_on_breaks: bool,
    pub bounce_on_edges: bool,
    pub trail_scale: f64,
    pub trail_end_scale: f64,
    pub trail_density: f64,
    pub trail_max_length: i32,
    pub trail_remove_speed: f64,
    pub glow_end_scale: f64,
    pub inner_length_mult: f64,
    pub additive_blending: bool,
    pub cursor_ripples: bool,
    pub smoke_enabled: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct CursorColors {
    pub enable_rainbow: bool,
    pub rainbow_speed: f64,
    pub base_color: Hsv,
    pub enable_custom_hue_offset: bool,
    pub hue_offset: f64,
    pub flash_to_the_beat: bool,
    pub flash_amplitude: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Objects {
    pub draw_approach_circles: bool,
    pub draw_combo_numbers: bool,
    pub draw_follow_points: bool,
    pub load_spinners: bool,
    pub scale_to_the_beat: bool,
    pub stack_enabled: bool,
    pub sliders: Sliders,
    pub colors: Colors,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Sliders {
    pub force_slider_ball_texture: bool,
    pub draw_end_circles: bool,
    pub draw_slider_follow_circle: bool,
    pub draw_score_points: bool,
    pub slider_merge: bool,
    pub border_width: i32,
    pub distortions: SliderDistortions,
    pub snaking: Snaking,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct SliderDistortions {
    enabled: bool,
    viewport_size: u64,
    use_custom_resolution: bool,
    custom_resolution_x: u64,
    custom_resolution_y: u64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Snaking {
    #[serde(rename = "In")]
    pub in_field: bool,
    pub out: bool,
    pub out_fade_instant: bool,
    pub duration_multiplier: f64,
    pub fade_multiplier: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Colors {
    pub mandala_textures_trigger: i32,
    pub mandala_textures_alpha: f64,
    pub color: ObjectColor,
    pub use_combo_colors: bool,
    pub combo_colors: Vec<Hsv>,
    pub use_skin_combo_colors: bool,
    pub use_beatmap_combo_colors: bool,
    pub sliders: ObjectSliders,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct ObjectSliders {
    pub white_score_points: bool,
    pub score_point_color_offset: i32,
    pub slider_ball_tint: bool,
    pub border: Border,
    pub body: Body,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Border {
    pub use_hit_circle_color: bool,
    pub color: ObjectColor,
    pub enable_custom_gradient_offset: bool,
    pub custom_gradient_offset: i32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Body {
    pub use_hit_circle_color: bool,
    pub color: ObjectColor,
    pub inner_offset: f64,
    pub outer_offset: f64,
    pub inner_alpha: f64,
    pub outer_alpha: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct ObjectColor {
    pub enable_rainbow: bool,
    pub rainbow_speed: f64,
    pub base_color: Hsv,
    pub enable_custom_hue_offset: bool,
    pub hue_offset: f64,
    pub flash_to_the_beat: bool,
    pub flash_amplitude: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Playfield {
    pub draw_objects: bool,
    pub draw_cursors: bool,
    pub scale: f64,
    pub osu_shift: bool,
    pub shift_y: f64,
    pub shift_x: f64,
    pub scale_storyboard_with_playfield: bool,
    pub move_storyboard_with_playfield: bool,
    pub lead_in_time: f64,
    pub lead_in_hold: f64,
    pub fade_out_time: f64,
    pub seizure_warning: SeizureWarning,
    pub background: Background,
    pub logo: Logo,
    pub bloom: Bloom,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct SeizureWarning {
    pub enabled: bool,
    pub duration: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Background {
    pub load_storyboards: bool,
    pub load_videos: bool,
    pub flash_to_the_beat: bool,
    pub dim: Dim,
    pub parallax: Parallax,
    pub blur: Blur,
    pub triangles: Triangles,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Dim {
    pub intro: f64,
    pub normal: f64,
    pub breaks: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Parallax {
    pub enabled: bool,
    pub amount: f64,
    pub speed: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Blur {
    pub enabled: bool,
    pub values: BlurValues,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct BlurValues {
    pub intro: f64,
    pub normal: f64,
    pub breaks: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Triangles {
    pub enabled: bool,
    pub shadowed: bool,
    pub draw_over_blur: bool,
    pub parallax_multiplier: f64,
    pub density: f64,
    pub scale: f64,
    pub speed: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Logo {
    pub enabled: bool,
    pub draw_spectrum: bool,
    pub dim: Dim,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Bloom {
    pub enabled: bool,
    pub bloom_to_the_beat: bool,
    pub bloom_beat_addition: f64,
    pub threshold: f64,
    pub blur: f64,
    pub power: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct CursorDance {
    pub movers: Vec<Mover>,
    pub spinners: Vec<Spinner>,
    pub combo_tag: bool,
    pub battle: bool,
    pub do_spinners_together: bool,
    #[serde(rename = "TAGSliderDance")]
    pub tag_slider_dance: bool,
    pub mover_settings: MoverSettings,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Mover {
    pub mover: String,
    pub slider_dance: bool,
    pub random_slider_dance: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Spinner {
    pub mover: String,
    pub center_offset_x: f64,
    pub center_offset_y: f64,
    pub radius: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct MoverSettings {
    pub bezier: Vec<Bezier>,
    pub flower: Vec<Flower>,
    pub half_circle: Vec<HalfCircle>,
    pub spline: Vec<Spline>,
    pub momentum: Vec<Momentum>,
    pub ex_gon: Vec<ExGon>,
    pub linear: Vec<Linear>,
    pub pippi: Vec<Pippi>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Bezier {
    pub aggressiveness: f64,
    pub slider_aggressiveness: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Flower {
    pub angle_offset: f64,
    pub distance_mult: f64,
    pub stream_angle_offset: f64,
    pub long_jump: i32,
    pub long_jump_mult: f64,
    pub long_jump_on_equal_pos: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct HalfCircle {
    pub radius_multiplier: f64,
    pub stream_trigger: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Spline {
    pub rotational_force: bool,
    pub stream_half_circle: bool,
    pub stream_wobble: bool,
    pub wobble_scale: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Momentum {
    pub skip_stack_angles: bool,
    pub stream_restrict: bool,
    pub duration_mult: f64,
    pub duration_trigger: f64,
    pub stream_mult: f64,
    pub restrict_angle: f64,
    pub restrict_area: f64,
    pub restrict_invert: bool,
    pub distance_mult: f64,
    pub distance_mult_out: f64,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct ExGon {
    pub delay: u32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Linear {
    pub wait_for_preempt: bool,
    pub reaction_time: f64,
    pub choppy_long_objects: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Pippi {
    pub rotation_speed: f64,
    pub radius_multiplier: f64,
    pub spinner_radius: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Knockout {
    pub mode: u8,
    pub grace_end_time: i32,
    pub bubble_minimum_combo: u32,
    pub exclude_mods: String,
    pub hide_mods: String,
    pub max_players: u32,
    pub min_players: u32,
    pub revive_players_at_end: bool,
    pub live_sort: bool,
    pub sort_by: String,
    pub hide_overlay_on_breaks: bool,
    pub min_cursor_size: f64,
    pub max_cursor_size: f64,
    pub add_danser: bool,
    pub danser_name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Recording {
    pub frame_width: i32,
    pub frame_height: i32,
    #[serde(rename = "FPS")]
    pub fps: i32,
    #[serde(rename = "EncodingFPSCap")]
    pub encoding_fpscap: i32,
    pub encoder: String,
    #[serde(rename = "libx264")]
    pub libx264: Libx264,
    #[serde(rename = "libx265")]
    pub libx265: Libx265,
    #[serde(rename = "h264_nvenc")]
    pub h264_nvenc: H264Nvenc,
    #[serde(rename = "hevc_nvenc")]
    pub hevc_nvenc: HevcNvenc,
    #[serde(rename = "h264_qsv")]
    pub h264_qsv: H264Qsv,
    #[serde(rename = "hevc_qsv")]
    pub hevc_qsv: HevcQsv,
    #[serde(rename = "custom")]
    pub custom: Custom,
    pub pixel_format: String,
    pub filters: String,
    pub audio_codec: String,
    #[serde(rename = "aac")]
    pub aac: Aac,
    #[serde(rename = "libmp3lame")]
    pub libmp3lame: Libmp3lame,
    #[serde(rename = "libopus")]
    pub libopus: Libopus,
    #[serde(rename = "flac")]
    pub flac: Flac,
    #[serde(rename = "customAudio")]
    pub custom_audio: CustomAudio,
    pub audio_filters: String,
    pub output_dir: String,
    pub container: String,
    #[serde(rename = "ShowFFmpegLogs")]
    pub show_ffmpeg_logs: bool,
    pub motion_blur: MotionBlur,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Libx264 {
    pub rate_control: String,
    pub bitrate: String,
    #[serde(rename = "CRF")]
    pub crf: i64,
    pub profile: String,
    pub preset: String,
    pub additional_options: String,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Libx265 {
    pub rate_control: String,
    pub bitrate: String,
    #[serde(rename = "CRF")]
    pub crf: i64,
    pub preset: String,
    pub additional_options: String,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct H264Nvenc {
    pub rate_control: String,
    pub bitrate: String,
    #[serde(rename = "CQ")]
    pub cq: i64,
    pub profile: String,
    pub preset: String,
    pub additional_options: String,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct HevcNvenc {
    pub rate_control: String,
    pub bitrate: String,
    #[serde(rename = "CQ")]
    pub cq: i64,
    pub preset: String,
    pub additional_options: String,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct H264Qsv {
    pub rate_control: String,
    pub bitrate: String,
    pub quality: i64,
    pub profile: String,
    pub preset: String,
    pub additional_options: String,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct HevcQsv {
    pub rate_control: String,
    pub bitrate: String,
    pub quality: i64,
    pub preset: String,
    pub additional_options: String,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Custom {
    pub custom_options: String,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Aac {
    pub bitrate: String,
    pub additional_options: String,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Libmp3lame {
    pub rate_control: String,
    pub target_bitrate: String,
    pub additional_options: String,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Libopus {
    pub rate_control: String,
    pub target_bitrate: String,
    pub additional_options: String,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Flac {
    pub compression_level: i64,
    pub additional_options: String,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct CustomAudio {
    pub custom_options: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct MotionBlur {
    pub enabled: bool,
    pub oversample_multiplier: u32,
    pub blend_frames: u32,
    #[serde(rename = "BlendFunctionID")]
    pub blend_function_id: u32,
    pub gauss_weights_mult: f64,
}
