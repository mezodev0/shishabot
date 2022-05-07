use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Settings {
    pub general: General,
    pub graphics: Graphics,
    pub audio: Audio,
    pub input: Input,
    pub gameplay: Gameplay,
    pub skin: Skin,
    pub cursor: Cursor2,
    pub objects: Objects,
    pub playfield: Playfield,
    pub cursor_dance: CursorDance,
    pub knockout: Knockout,
    pub recording: Recording,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct General {
    pub osu_songs_dir: String,
    pub osu_skins_dir: String,
    pub discord_presence_on: bool,
    pub unpack_osz_files: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Graphics {
    pub width: i64,
    pub height: i64,
    pub window_width: i64,
    pub window_height: i64,
    pub fullscreen: bool,
    #[serde(rename = "VSync")]
    pub vsync: bool,
    #[serde(rename = "FPSCap")]
    pub fpscap: i64,
    #[serde(rename = "MSAA")]
    pub msaa: i64,
    #[serde(rename = "ShowFPS")]
    pub show_fps: bool,
    pub experimental: Experimental,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Experimental {
    pub use_persistent_buffers: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Audio {
    pub general_volume: f64,
    pub music_volume: f64,
    pub sample_volume: f64,
    pub offset: i64,
    pub hitsound_position_multiplier: i64,
    pub ignore_beatmap_samples: bool,
    pub ignore_beatmap_sample_volume: bool,
    pub play_nightcore_samples: bool,
    pub beat_scale: f64,
    pub beat_use_timing_points: bool,
    #[serde(rename = "Linux/Unix")]
    pub linux_unix: LinuxUnix,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LinuxUnix {
    pub bass_playback_buffer_length: i64,
    pub bass_device_buffer_length: i64,
    pub bass_update_period: i64,
    pub bass_device_update_period: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Input {
    pub left_key: String,
    pub right_key: String,
    pub restart_key: String,
    pub smoke_key: String,
    pub mouse_buttons_disabled: bool,
    pub mouse_high_precision: bool,
    pub mouse_sensitivity: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Gameplay {
    pub hit_error_meter: HitErrorMeter,
    pub aim_error_meter: AimErrorMeter,
    pub score: Score,
    pub hp_bar: Container,
    pub combo_counter: Container,
    #[serde(rename = "PPCounter")]
    pub pp_counter: Ppcounter,
    pub hit_counter: HitCounter,
    pub key_overlay: Container,
    pub score_board: ScoreBoard,
    pub mods: Mods,
    pub boundaries: Boundaries,
    pub show_results_screen: bool,
    pub results_screen_time: i64,
    pub results_use_local_time_zone: bool,
    pub show_warning_arrows: bool,
    pub show_hit_lighting: bool,
    pub flashlight_dim: i64,
    pub play_username: String,
    #[serde(rename = "UseLazerPP")]
    pub use_lazer_pp: bool,
    pub strain_graph: StrainGraph,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct HitErrorMeter {
    pub show: bool,
    pub scale: u64,
    pub opacity: i64,
    #[serde(rename = "XOffset")]
    pub xoffset: i64,
    #[serde(rename = "YOffset")]
    pub yoffset: i64,
    pub show_positional_misses: bool,
    pub show_unstable_rate: bool,
    pub unstable_rate_decimals: i64,
    pub unstable_rate_scale: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AimErrorMeter {
    pub show: bool,
    pub scale: u64,
    pub opacity: i64,
    #[serde(rename = "XPosition")]
    pub xposition: i64,
    #[serde(rename = "YPosition")]
    pub yposition: i64,
    pub dot_scale: i64,
    pub align: String,
    pub show_unstable_rate: bool,
    pub unstable_rate_scale: i64,
    pub unstable_rate_decimals: u64,
    pub cap_positional_misses: bool,
    pub angle_normalized: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Score {
    pub show: bool,
    pub scale: i64,
    pub opacity: i64,
    #[serde(rename = "XOffset")]
    pub xoffset: i64,
    #[serde(rename = "YOffset")]
    pub yoffset: i64,
    pub progress_bar: String,
    pub show_grade_always: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Container {
    pub show: bool,
    pub scale: i64,
    pub opacity: i64,
    #[serde(rename = "XOffset")]
    pub xoffset: i64,
    #[serde(rename = "YOffset")]
    pub yoffset: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Ppcounter {
    pub show: bool,
    pub scale: i64,
    pub opacity: i64,
    pub color: Color,
    #[serde(rename = "XPosition")]
    pub xposition: i64,
    #[serde(rename = "YPosition")]
    pub yposition: i64,
    pub decimals: u64,
    pub align: String,
    pub show_in_results: bool,
    #[serde(rename = "ShowPPComponents")]
    pub show_ppcomponents: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Color {
    pub hue: f64,
    pub saturation: f64,
    pub value: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct HitCounter {
    pub show: bool,
    pub scale: f64,
    pub opacity: i64,
    pub color: Vec<Color>,
    #[serde(rename = "XPosition")]
    pub xposition: i64,
    #[serde(rename = "YPosition")]
    pub yposition: i64,
    pub spacing: i64,
    pub font_scale: i64,
    pub align: String,
    pub value_align: String,
    pub vertical: bool,
    #[serde(rename = "Show300")]
    pub show300: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ScoreBoard {
    pub show: bool,
    pub scale: i64,
    pub opacity: i64,
    #[serde(rename = "XOffset")]
    pub xoffset: i64,
    #[serde(rename = "YOffset")]
    pub yoffset: i64,
    pub align_right: bool,
    pub hide_others: bool,
    pub show_avatars: bool,
    pub explosion_scale: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Mods {
    pub show: bool,
    pub scale: i64,
    pub opacity: i64,
    #[serde(rename = "XOffset")]
    pub xoffset: i64,
    #[serde(rename = "YOffset")]
    pub yoffset: i64,
    pub hide_in_replays: bool,
    pub fold_in_replays: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Boundaries {
    pub enabled: bool,
    pub border_thickness: i64,
    pub border_fill: i64,
    pub border_color: Color,
    pub border_opacity: i64,
    pub background_color: Color,
    pub background_opacity: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Skin {
    pub current_skin: String,
    pub fallback_skin: String,
    pub use_colors_from_skin: bool,
    pub use_beatmap_colors: bool,
    pub cursor: Cursor,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Cursor {
    pub use_skin_cursor: bool,
    pub scale: f64,
    pub force_long_trail: bool,
    pub long_trail_length: i64,
    pub long_trail_density: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Cursor2 {
    pub trail_style: i64,
    #[serde(rename = "Style23Speed")]
    pub style23speed: f64,
    #[serde(rename = "Style4Shift")]
    pub style4shift: f64,
    pub colors: Colors,
    pub enable_custom_tag_color_offset: bool,
    pub tag_color_offset: i64,
    pub enable_trail_glow: bool,
    pub enable_custom_trail_glow_offset: bool,
    pub trail_glow_offset: i64,
    #[serde(rename = "ScaleToCS")]
    pub scale_to_cs: bool,
    pub cursor_size: i64,
    pub cursor_expand: bool,
    pub scale_to_the_beat: bool,
    pub show_cursors_on_breaks: bool,
    pub bounce_on_edges: bool,
    pub trail_scale: i64,
    pub trail_end_scale: f64,
    pub trail_density: f64,
    pub trail_max_length: i64,
    pub trail_remove_speed: i64,
    pub glow_end_scale: f64,
    pub inner_length_mult: f64,
    pub additive_blending: bool,
    pub cursor_ripples: bool,
    pub smoke_enabled: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Colors {
    pub enable_rainbow: bool,
    pub rainbow_speed: i64,
    pub base_color: Color,
    pub enable_custom_hue_offset: bool,
    pub hue_offset: i64,
    pub flash_to_the_beat: bool,
    pub flash_amplitude: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Objects {
    pub draw_approach_circles: bool,
    pub draw_combo_numbers: bool,
    pub draw_follow_points: bool,
    pub load_spinners: bool,
    pub scale_to_the_beat: bool,
    pub stack_enabled: bool,
    pub sliders: Sliders,
    pub colors: Colors2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Sliders {
    pub force_slider_ball_texture: bool,
    pub draw_end_circles: bool,
    pub draw_slider_follow_circle: bool,
    pub draw_score_points: bool,
    pub slider_merge: bool,
    pub slider_distortions: bool,
    pub border_width: i64,
    pub quality: Quality,
    pub snaking: Snaking,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Quality {
    pub circle_level_of_detail: i64,
    pub path_level_of_detail: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Snaking {
    #[serde(rename = "In")]
    pub in_field: bool,
    pub out: bool,
    pub out_fade_instant: bool,
    pub duration_multiplier: i64,
    pub fade_multiplier: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Colors2 {
    pub mandala_textures_trigger: i64,
    pub mandala_textures_alpha: f64,
    pub color: Colors,
    pub use_combo_colors: bool,
    pub combo_colors: Vec<Color>,
    pub use_skin_combo_colors: bool,
    pub use_beatmap_combo_colors: bool,
    pub sliders: Sliders2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Sliders2 {
    pub white_score_points: bool,
    pub score_point_color_offset: i64,
    pub slider_ball_tint: bool,
    pub border: Border,
    pub body: Body,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Border {
    pub use_hit_circle_color: bool,
    pub color: Colors,
    pub enable_custom_gradient_offset: bool,
    pub custom_gradient_offset: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Body {
    pub use_hit_circle_color: bool,
    pub color: Colors,
    pub inner_offset: f64,
    pub outer_offset: f64,
    pub inner_alpha: f64,
    pub outer_alpha: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Playfield {
    pub draw_objects: bool,
    pub draw_cursors: bool,
    pub scale: i64,
    pub osu_shift: bool,
    pub shift_y: i64,
    pub shift_x: i64,
    pub scale_storyboard_with_playfield: bool,
    pub lead_in_time: i64,
    pub lead_in_hold: i64,
    pub fade_out_time: i64,
    pub seizure_warning: SeizureWarning,
    pub background: Background,
    pub logo: Logo,
    pub bloom: Bloom,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SeizureWarning {
    pub enabled: bool,
    pub duration: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Background {
    pub load_storyboards: bool,
    pub load_videos: bool,
    pub flash_to_the_beat: bool,
    pub dim: Alpha,
    pub parallax: Parallax,
    pub blur: Blur,
    pub triangles: Triangles,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Alpha {
    pub intro: f64,
    pub normal: f64,
    pub breaks: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Parallax {
    pub amount: i64,
    pub speed: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Blur {
    pub enabled: bool,
    pub values: Alpha,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Triangles {
    pub enabled: bool,
    pub shadowed: bool,
    pub draw_over_blur: bool,
    pub parallax_multiplier: i64,
    pub density: i64,
    pub scale: i64,
    pub speed: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Logo {
    pub draw_spectrum: bool,
    pub dim: Alpha,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Bloom {
    pub enabled: bool,
    pub bloom_to_the_beat: bool,
    pub bloom_beat_addition: f64,
    pub threshold: f64,
    pub blur: f64,
    pub power: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CursorDance {
    pub movers: Vec<Mover>,
    pub spinners: Vec<Spinner>,
    pub combo_tag: bool,
    pub battle: bool,
    pub do_spinners_together: bool,
    #[serde(rename = "TAGSliderDance")]
    pub tagslider_dance: bool,
    pub mover_settings: MoverSettings,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Mover {
    pub mover: String,
    pub slider_dance: bool,
    pub random_slider_dance: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Spinner {
    pub mover: String,
    pub radius: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
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
#[serde(rename_all = "PascalCase")]
pub struct Bezier {
    pub aggressiveness: i64,
    pub slider_aggressiveness: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Flower {
    pub angle_offset: i64,
    pub distance_mult: f64,
    pub stream_angle_offset: i64,
    pub long_jump: i64,
    pub long_jump_mult: f64,
    pub long_jump_on_equal_pos: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct HalfCircle {
    pub radius_multiplier: i64,
    pub stream_trigger: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Spline {
    pub rotational_force: bool,
    pub stream_half_circle: bool,
    pub stream_wobble: bool,
    pub wobble_scale: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Momentum {
    pub skip_stack_angles: bool,
    pub stream_restrict: bool,
    pub duration_mult: i64,
    pub duration_trigger: i64,
    pub stream_mult: f64,
    pub restrict_angle: i64,
    pub restrict_area: i64,
    pub restrict_invert: bool,
    pub distance_mult: f64,
    pub distance_mult_out: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ExGon {
    pub delay: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Linear {
    pub wait_for_preempt: bool,
    pub reaction_time: i64,
    pub choppy_long_objects: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Pippi {
    pub rotation_speed: f64,
    pub radius_multiplier: f64,
    pub spinner_radius: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Knockout {
    pub mode: i64,
    pub exclude_mods: String,
    pub hide_mods: String,
    pub max_players: i64,
    pub bubble_minimum_combo: i64,
    pub revive_players_at_end: bool,
    pub live_sort: bool,
    pub sort_by: String,
    pub hide_overlay_on_breaks: bool,
    pub min_cursor_size: i64,
    pub max_cursor_size: i64,
    pub add_danser: bool,
    pub danser_name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Recording {
    pub frame_width: i64,
    pub frame_height: i64,
    #[serde(rename = "FPS")]
    pub fps: i64,
    #[serde(rename = "EncodingFPSCap")]
    pub encoding_fpscap: i64,
    pub encoder: String,
    pub encoder_options: String,
    pub profile: String,
    pub preset: String,
    pub pixel_format: String,
    pub filters: String,
    pub audio_codec: String,
    pub audio_options: String,
    pub audio_filters: String,
    pub output_dir: String,
    pub container: String,
    #[serde(rename = "ShowFFmpegLogs")]
    pub show_ffmpeg_logs: bool,
    pub motion_blur: MotionBlur,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MotionBlur {
    pub enabled: bool,
    pub oversample_multiplier: i64,
    pub blend_frames: i64,
    pub blend_weights: BlendWeights,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BlendWeights {
    pub use_manual_weights: bool,
    pub manual_weights: String,
    #[serde(rename = "AutoWeightsID")]
    pub auto_weights_id: i64,
    pub gauss_weights_mult: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StrainGraph {
    pub show: bool,
    pub opacity: i64,
    pub x_position: i64,
    pub y_position: i64,
    pub align: String,
    pub width: i64,
    pub height: i64,
    pub bg_color: Color,
    pub fg_color: Color,
}
