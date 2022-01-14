use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(rename = "General")]
    pub general: General,
    #[serde(rename = "Graphics")]
    pub graphics: Graphics,
    #[serde(rename = "Audio")]
    pub audio: Audio,
    #[serde(rename = "Input")]
    pub input: Input,
    #[serde(rename = "Gameplay")]
    pub gameplay: Gameplay,
    #[serde(rename = "Skin")]
    pub skin: Skin,
    #[serde(rename = "Cursor")]
    pub cursor: Cursor2,
    #[serde(rename = "Objects")]
    pub objects: Objects,
    #[serde(rename = "Playfield")]
    pub playfield: Playfield,
    #[serde(rename = "CursorDance")]
    pub cursor_dance: CursorDance,
    #[serde(rename = "Knockout")]
    pub knockout: Knockout,
    #[serde(rename = "Recording")]
    pub recording: Recording,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct General {
    #[serde(rename = "OsuSongsDir")]
    pub osu_songs_dir: String,
    #[serde(rename = "OsuSkinsDir")]
    pub osu_skins_dir: String,
    #[serde(rename = "DiscordPresenceOn")]
    pub discord_presence_on: bool,
    #[serde(rename = "UnpackOszFiles")]
    pub unpack_osz_files: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Graphics {
    #[serde(rename = "Width")]
    pub width: i64,
    #[serde(rename = "Height")]
    pub height: i64,
    #[serde(rename = "WindowWidth")]
    pub window_width: i64,
    #[serde(rename = "WindowHeight")]
    pub window_height: i64,
    #[serde(rename = "Fullscreen")]
    pub fullscreen: bool,
    #[serde(rename = "VSync")]
    pub vsync: bool,
    #[serde(rename = "FPSCap")]
    pub fpscap: i64,
    #[serde(rename = "MSAA")]
    pub msaa: i64,
    #[serde(rename = "ShowFPS")]
    pub show_fps: bool,
    #[serde(rename = "Experimental")]
    pub experimental: Experimental,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Experimental {
    #[serde(rename = "UsePersistentBuffers")]
    pub use_persistent_buffers: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Audio {
    #[serde(rename = "GeneralVolume")]
    pub general_volume: f64,
    #[serde(rename = "MusicVolume")]
    pub music_volume: f64,
    #[serde(rename = "SampleVolume")]
    pub sample_volume: f64,
    #[serde(rename = "Offset")]
    pub offset: i64,
    #[serde(rename = "HitsoundPositionMultiplier")]
    pub hitsound_position_multiplier: i64,
    #[serde(rename = "IgnoreBeatmapSamples")]
    pub ignore_beatmap_samples: bool,
    #[serde(rename = "IgnoreBeatmapSampleVolume")]
    pub ignore_beatmap_sample_volume: bool,
    #[serde(rename = "PlayNightcoreSamples")]
    pub play_nightcore_samples: bool,
    #[serde(rename = "BeatScale")]
    pub beat_scale: f64,
    #[serde(rename = "BeatUseTimingPoints")]
    pub beat_use_timing_points: bool,
    #[serde(rename = "Linux/Unix")]
    pub linux_unix: LinuxUnix,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinuxUnix {
    #[serde(rename = "BassPlaybackBufferLength")]
    pub bass_playback_buffer_length: i64,
    #[serde(rename = "BassDeviceBufferLength")]
    pub bass_device_buffer_length: i64,
    #[serde(rename = "BassUpdatePeriod")]
    pub bass_update_period: i64,
    #[serde(rename = "BassDeviceUpdatePeriod")]
    pub bass_device_update_period: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    #[serde(rename = "LeftKey")]
    pub left_key: String,
    #[serde(rename = "RightKey")]
    pub right_key: String,
    #[serde(rename = "RestartKey")]
    pub restart_key: String,
    #[serde(rename = "SmokeKey")]
    pub smoke_key: String,
    #[serde(rename = "MouseButtonsDisabled")]
    pub mouse_buttons_disabled: bool,
    #[serde(rename = "MouseHighPrecision")]
    pub mouse_high_precision: bool,
    #[serde(rename = "MouseSensitivity")]
    pub mouse_sensitivity: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Gameplay {
    #[serde(rename = "HitErrorMeter")]
    pub hit_error_meter: HitErrorMeter,
    #[serde(rename = "AimErrorMeter")]
    pub aim_error_meter: AimErrorMeter,
    #[serde(rename = "Score")]
    pub score: Score,
    #[serde(rename = "HpBar")]
    pub hp_bar: HpBar,
    #[serde(rename = "ComboCounter")]
    pub combo_counter: ComboCounter,
    #[serde(rename = "PPCounter")]
    pub pp_counter: Ppcounter,
    #[serde(rename = "HitCounter")]
    pub hit_counter: HitCounter,
    #[serde(rename = "KeyOverlay")]
    pub key_overlay: KeyOverlay,
    #[serde(rename = "ScoreBoard")]
    pub score_board: ScoreBoard,
    #[serde(rename = "Mods")]
    pub mods: Mods,
    #[serde(rename = "Boundaries")]
    pub boundaries: Boundaries,
    #[serde(rename = "ShowResultsScreen")]
    pub show_results_screen: bool,
    #[serde(rename = "ResultsScreenTime")]
    pub results_screen_time: i64,
    #[serde(rename = "ResultsUseLocalTimeZone")]
    pub results_use_local_time_zone: bool,
    #[serde(rename = "ShowWarningArrows")]
    pub show_warning_arrows: bool,
    #[serde(rename = "ShowHitLighting")]
    pub show_hit_lighting: bool,
    #[serde(rename = "FlashlightDim")]
    pub flashlight_dim: i64,
    #[serde(rename = "PlayUsername")]
    pub play_username: String,
    #[serde(rename = "UseLazerPP")]
    pub use_lazer_pp: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HitErrorMeter {
    #[serde(rename = "Show")]
    pub show: bool,
    #[serde(rename = "Scale")]
    pub scale: u64,
    #[serde(rename = "Opacity")]
    pub opacity: i64,
    #[serde(rename = "XOffset")]
    pub xoffset: i64,
    #[serde(rename = "YOffset")]
    pub yoffset: i64,
    #[serde(rename = "ShowPositionalMisses")]
    pub show_positional_misses: bool,
    #[serde(rename = "ShowUnstableRate")]
    pub show_unstable_rate: bool,
    #[serde(rename = "UnstableRateDecimals")]
    pub unstable_rate_decimals: i64,
    #[serde(rename = "UnstableRateScale")]
    pub unstable_rate_scale: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AimErrorMeter {
    #[serde(rename = "Show")]
    pub show: bool,
    #[serde(rename = "Scale")]
    pub scale: u64,
    #[serde(rename = "Opacity")]
    pub opacity: i64,
    #[serde(rename = "XPosition")]
    pub xposition: i64,
    #[serde(rename = "YPosition")]
    pub yposition: i64,
    #[serde(rename = "DotScale")]
    pub dot_scale: i64,
    #[serde(rename = "Align")]
    pub align: String,
    #[serde(rename = "ShowUnstableRate")]
    pub show_unstable_rate: bool,
    #[serde(rename = "UnstableRateScale")]
    pub unstable_rate_scale: i64,
    #[serde(rename = "UnstableRateDecimals")]
    pub unstable_rate_decimals: u64,
    #[serde(rename = "CapPositionalMisses")]
    pub cap_positional_misses: bool,
    #[serde(rename = "AngleNormalized")]
    pub angle_normalized: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Score {
    #[serde(rename = "Show")]
    pub show: bool,
    #[serde(rename = "Scale")]
    pub scale: i64,
    #[serde(rename = "Opacity")]
    pub opacity: i64,
    #[serde(rename = "XOffset")]
    pub xoffset: i64,
    #[serde(rename = "YOffset")]
    pub yoffset: i64,
    #[serde(rename = "ProgressBar")]
    pub progress_bar: String,
    #[serde(rename = "ShowGradeAlways")]
    pub show_grade_always: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HpBar {
    #[serde(rename = "Show")]
    pub show: bool,
    #[serde(rename = "Scale")]
    pub scale: i64,
    #[serde(rename = "Opacity")]
    pub opacity: i64,
    #[serde(rename = "XOffset")]
    pub xoffset: i64,
    #[serde(rename = "YOffset")]
    pub yoffset: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComboCounter {
    #[serde(rename = "Show")]
    pub show: bool,
    #[serde(rename = "Scale")]
    pub scale: i64,
    #[serde(rename = "Opacity")]
    pub opacity: i64,
    #[serde(rename = "XOffset")]
    pub xoffset: i64,
    #[serde(rename = "YOffset")]
    pub yoffset: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ppcounter {
    #[serde(rename = "Show")]
    pub show: bool,
    #[serde(rename = "Scale")]
    pub scale: i64,
    #[serde(rename = "Opacity")]
    pub opacity: i64,
    #[serde(rename = "Color")]
    pub color: Color,
    #[serde(rename = "XPosition")]
    pub xposition: i64,
    #[serde(rename = "YPosition")]
    pub yposition: i64,
    #[serde(rename = "Decimals")]
    pub decimals: u64,
    #[serde(rename = "Align")]
    pub align: String,
    #[serde(rename = "ShowInResults")]
    pub show_in_results: bool,
    #[serde(rename = "ShowPPComponents")]
    pub show_ppcomponents: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Color {
    #[serde(rename = "Hue")]
    pub hue: i64,
    #[serde(rename = "Saturation")]
    pub saturation: i64,
    #[serde(rename = "Value")]
    pub value: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HitCounter {
    #[serde(rename = "Show")]
    pub show: bool,
    #[serde(rename = "Scale")]
    pub scale: f64,
    #[serde(rename = "Opacity")]
    pub opacity: i64,
    #[serde(rename = "Color")]
    pub color: Vec<Color2>,
    #[serde(rename = "XPosition")]
    pub xposition: i64,
    #[serde(rename = "YPosition")]
    pub yposition: i64,
    #[serde(rename = "Spacing")]
    pub spacing: i64,
    #[serde(rename = "FontScale")]
    pub font_scale: i64,
    #[serde(rename = "Align")]
    pub align: String,
    #[serde(rename = "ValueAlign")]
    pub value_align: String,
    #[serde(rename = "Vertical")]
    pub vertical: bool,
    #[serde(rename = "Show300")]
    pub show300: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Color2 {
    #[serde(rename = "Hue")]
    pub hue: i64,
    #[serde(rename = "Saturation")]
    pub saturation: f64,
    #[serde(rename = "Value")]
    pub value: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyOverlay {
    #[serde(rename = "Show")]
    pub show: bool,
    #[serde(rename = "Scale")]
    pub scale: i64,
    #[serde(rename = "Opacity")]
    pub opacity: i64,
    #[serde(rename = "XOffset")]
    pub xoffset: i64,
    #[serde(rename = "YOffset")]
    pub yoffset: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreBoard {
    #[serde(rename = "Show")]
    pub show: bool,
    #[serde(rename = "Scale")]
    pub scale: i64,
    #[serde(rename = "Opacity")]
    pub opacity: i64,
    #[serde(rename = "XOffset")]
    pub xoffset: i64,
    #[serde(rename = "YOffset")]
    pub yoffset: i64,
    #[serde(rename = "AlignRight")]
    pub align_right: bool,
    #[serde(rename = "HideOthers")]
    pub hide_others: bool,
    #[serde(rename = "ShowAvatars")]
    pub show_avatars: bool,
    #[serde(rename = "ExplosionScale")]
    pub explosion_scale: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mods {
    #[serde(rename = "Show")]
    pub show: bool,
    #[serde(rename = "Scale")]
    pub scale: i64,
    #[serde(rename = "Opacity")]
    pub opacity: i64,
    #[serde(rename = "XOffset")]
    pub xoffset: i64,
    #[serde(rename = "YOffset")]
    pub yoffset: i64,
    #[serde(rename = "HideInReplays")]
    pub hide_in_replays: bool,
    #[serde(rename = "FoldInReplays")]
    pub fold_in_replays: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Boundaries {
    #[serde(rename = "Enabled")]
    pub enabled: bool,
    #[serde(rename = "BorderThickness")]
    pub border_thickness: i64,
    #[serde(rename = "BorderFill")]
    pub border_fill: i64,
    #[serde(rename = "BorderColor")]
    pub border_color: BorderColor,
    #[serde(rename = "BorderOpacity")]
    pub border_opacity: i64,
    #[serde(rename = "BackgroundColor")]
    pub background_color: BackgroundColor,
    #[serde(rename = "BackgroundOpacity")]
    pub background_opacity: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BorderColor {
    #[serde(rename = "Hue")]
    pub hue: i64,
    #[serde(rename = "Saturation")]
    pub saturation: i64,
    #[serde(rename = "Value")]
    pub value: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackgroundColor {
    #[serde(rename = "Hue")]
    pub hue: i64,
    #[serde(rename = "Saturation")]
    pub saturation: i64,
    #[serde(rename = "Value")]
    pub value: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skin {
    #[serde(rename = "CurrentSkin")]
    pub current_skin: String,
    #[serde(rename = "FallbackSkin")]
    pub fallback_skin: String,
    #[serde(rename = "UseColorsFromSkin")]
    pub use_colors_from_skin: bool,
    #[serde(rename = "UseBeatmapColors")]
    pub use_beatmap_colors: bool,
    #[serde(rename = "Cursor")]
    pub cursor: Cursor,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cursor {
    #[serde(rename = "UseSkinCursor")]
    pub use_skin_cursor: bool,
    #[serde(rename = "Scale")]
    pub scale: f64,
    #[serde(rename = "ForceLongTrail")]
    pub force_long_trail: bool,
    #[serde(rename = "LongTrailLength")]
    pub long_trail_length: i64,
    #[serde(rename = "LongTrailDensity")]
    pub long_trail_density: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cursor2 {
    #[serde(rename = "TrailStyle")]
    pub trail_style: i64,
    #[serde(rename = "Style23Speed")]
    pub style23speed: f64,
    #[serde(rename = "Style4Shift")]
    pub style4shift: f64,
    #[serde(rename = "Colors")]
    pub colors: Colors,
    #[serde(rename = "EnableCustomTagColorOffset")]
    pub enable_custom_tag_color_offset: bool,
    #[serde(rename = "TagColorOffset")]
    pub tag_color_offset: i64,
    #[serde(rename = "EnableTrailGlow")]
    pub enable_trail_glow: bool,
    #[serde(rename = "EnableCustomTrailGlowOffset")]
    pub enable_custom_trail_glow_offset: bool,
    #[serde(rename = "TrailGlowOffset")]
    pub trail_glow_offset: i64,
    #[serde(rename = "ScaleToCS")]
    pub scale_to_cs: bool,
    #[serde(rename = "CursorSize")]
    pub cursor_size: i64,
    #[serde(rename = "CursorExpand")]
    pub cursor_expand: bool,
    #[serde(rename = "ScaleToTheBeat")]
    pub scale_to_the_beat: bool,
    #[serde(rename = "ShowCursorsOnBreaks")]
    pub show_cursors_on_breaks: bool,
    #[serde(rename = "BounceOnEdges")]
    pub bounce_on_edges: bool,
    #[serde(rename = "TrailScale")]
    pub trail_scale: i64,
    #[serde(rename = "TrailEndScale")]
    pub trail_end_scale: f64,
    #[serde(rename = "TrailDensity")]
    pub trail_density: f64,
    #[serde(rename = "TrailMaxLength")]
    pub trail_max_length: i64,
    #[serde(rename = "TrailRemoveSpeed")]
    pub trail_remove_speed: i64,
    #[serde(rename = "GlowEndScale")]
    pub glow_end_scale: f64,
    #[serde(rename = "InnerLengthMult")]
    pub inner_length_mult: f64,
    #[serde(rename = "AdditiveBlending")]
    pub additive_blending: bool,
    #[serde(rename = "CursorRipples")]
    pub cursor_ripples: bool,
    #[serde(rename = "SmokeEnabled")]
    pub smoke_enabled: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Colors {
    #[serde(rename = "EnableRainbow")]
    pub enable_rainbow: bool,
    #[serde(rename = "RainbowSpeed")]
    pub rainbow_speed: i64,
    #[serde(rename = "BaseColor")]
    pub base_color: BaseColor,
    #[serde(rename = "EnableCustomHueOffset")]
    pub enable_custom_hue_offset: bool,
    #[serde(rename = "HueOffset")]
    pub hue_offset: i64,
    #[serde(rename = "FlashToTheBeat")]
    pub flash_to_the_beat: bool,
    #[serde(rename = "FlashAmplitude")]
    pub flash_amplitude: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseColor {
    #[serde(rename = "Hue")]
    pub hue: i64,
    #[serde(rename = "Saturation")]
    pub saturation: i64,
    #[serde(rename = "Value")]
    pub value: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Objects {
    #[serde(rename = "DrawApproachCircles")]
    pub draw_approach_circles: bool,
    #[serde(rename = "DrawComboNumbers")]
    pub draw_combo_numbers: bool,
    #[serde(rename = "DrawFollowPoints")]
    pub draw_follow_points: bool,
    #[serde(rename = "LoadSpinners")]
    pub load_spinners: bool,
    #[serde(rename = "ScaleToTheBeat")]
    pub scale_to_the_beat: bool,
    #[serde(rename = "StackEnabled")]
    pub stack_enabled: bool,
    #[serde(rename = "Sliders")]
    pub sliders: Sliders,
    #[serde(rename = "Colors")]
    pub colors: Colors2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sliders {
    #[serde(rename = "ForceSliderBallTexture")]
    pub force_slider_ball_texture: bool,
    #[serde(rename = "DrawEndCircles")]
    pub draw_end_circles: bool,
    #[serde(rename = "DrawSliderFollowCircle")]
    pub draw_slider_follow_circle: bool,
    #[serde(rename = "DrawScorePoints")]
    pub draw_score_points: bool,
    #[serde(rename = "SliderMerge")]
    pub slider_merge: bool,
    #[serde(rename = "SliderDistortions")]
    pub slider_distortions: bool,
    #[serde(rename = "BorderWidth")]
    pub border_width: i64,
    #[serde(rename = "Quality")]
    pub quality: Quality,
    #[serde(rename = "Snaking")]
    pub snaking: Snaking,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Quality {
    #[serde(rename = "CircleLevelOfDetail")]
    pub circle_level_of_detail: i64,
    #[serde(rename = "PathLevelOfDetail")]
    pub path_level_of_detail: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snaking {
    #[serde(rename = "In")]
    pub in_field: bool,
    #[serde(rename = "Out")]
    pub out: bool,
    #[serde(rename = "OutFadeInstant")]
    pub out_fade_instant: bool,
    #[serde(rename = "DurationMultiplier")]
    pub duration_multiplier: i64,
    #[serde(rename = "FadeMultiplier")]
    pub fade_multiplier: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Colors2 {
    #[serde(rename = "MandalaTexturesTrigger")]
    pub mandala_textures_trigger: i64,
    #[serde(rename = "MandalaTexturesAlpha")]
    pub mandala_textures_alpha: f64,
    #[serde(rename = "Color")]
    pub color: Color3,
    #[serde(rename = "UseComboColors")]
    pub use_combo_colors: bool,
    #[serde(rename = "ComboColors")]
    pub combo_colors: Vec<ComboColor>,
    #[serde(rename = "UseSkinComboColors")]
    pub use_skin_combo_colors: bool,
    #[serde(rename = "UseBeatmapComboColors")]
    pub use_beatmap_combo_colors: bool,
    #[serde(rename = "Sliders")]
    pub sliders: Sliders2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Color3 {
    #[serde(rename = "EnableRainbow")]
    pub enable_rainbow: bool,
    #[serde(rename = "RainbowSpeed")]
    pub rainbow_speed: i64,
    #[serde(rename = "BaseColor")]
    pub base_color: BaseColor2,
    #[serde(rename = "EnableCustomHueOffset")]
    pub enable_custom_hue_offset: bool,
    #[serde(rename = "HueOffset")]
    pub hue_offset: i64,
    #[serde(rename = "FlashToTheBeat")]
    pub flash_to_the_beat: bool,
    #[serde(rename = "FlashAmplitude")]
    pub flash_amplitude: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseColor2 {
    #[serde(rename = "Hue")]
    pub hue: i64,
    #[serde(rename = "Saturation")]
    pub saturation: i64,
    #[serde(rename = "Value")]
    pub value: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComboColor {
    #[serde(rename = "Hue")]
    pub hue: i64,
    #[serde(rename = "Saturation")]
    pub saturation: i64,
    #[serde(rename = "Value")]
    pub value: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sliders2 {
    #[serde(rename = "WhiteScorePoints")]
    pub white_score_points: bool,
    #[serde(rename = "ScorePointColorOffset")]
    pub score_point_color_offset: i64,
    #[serde(rename = "SliderBallTint")]
    pub slider_ball_tint: bool,
    #[serde(rename = "Border")]
    pub border: Border,
    #[serde(rename = "Body")]
    pub body: Body,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Border {
    #[serde(rename = "UseHitCircleColor")]
    pub use_hit_circle_color: bool,
    #[serde(rename = "Color")]
    pub color: Color4,
    #[serde(rename = "EnableCustomGradientOffset")]
    pub enable_custom_gradient_offset: bool,
    #[serde(rename = "CustomGradientOffset")]
    pub custom_gradient_offset: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Color4 {
    #[serde(rename = "EnableRainbow")]
    pub enable_rainbow: bool,
    #[serde(rename = "RainbowSpeed")]
    pub rainbow_speed: i64,
    #[serde(rename = "BaseColor")]
    pub base_color: BaseColor3,
    #[serde(rename = "EnableCustomHueOffset")]
    pub enable_custom_hue_offset: bool,
    #[serde(rename = "HueOffset")]
    pub hue_offset: i64,
    #[serde(rename = "FlashToTheBeat")]
    pub flash_to_the_beat: bool,
    #[serde(rename = "FlashAmplitude")]
    pub flash_amplitude: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseColor3 {
    #[serde(rename = "Hue")]
    pub hue: i64,
    #[serde(rename = "Saturation")]
    pub saturation: i64,
    #[serde(rename = "Value")]
    pub value: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Body {
    #[serde(rename = "UseHitCircleColor")]
    pub use_hit_circle_color: bool,
    #[serde(rename = "Color")]
    pub color: Color5,
    #[serde(rename = "InnerOffset")]
    pub inner_offset: f64,
    #[serde(rename = "OuterOffset")]
    pub outer_offset: f64,
    #[serde(rename = "InnerAlpha")]
    pub inner_alpha: f64,
    #[serde(rename = "OuterAlpha")]
    pub outer_alpha: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Color5 {
    #[serde(rename = "EnableRainbow")]
    pub enable_rainbow: bool,
    #[serde(rename = "RainbowSpeed")]
    pub rainbow_speed: i64,
    #[serde(rename = "BaseColor")]
    pub base_color: BaseColor4,
    #[serde(rename = "EnableCustomHueOffset")]
    pub enable_custom_hue_offset: bool,
    #[serde(rename = "HueOffset")]
    pub hue_offset: i64,
    #[serde(rename = "FlashToTheBeat")]
    pub flash_to_the_beat: bool,
    #[serde(rename = "FlashAmplitude")]
    pub flash_amplitude: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseColor4 {
    #[serde(rename = "Hue")]
    pub hue: i64,
    #[serde(rename = "Saturation")]
    pub saturation: i64,
    #[serde(rename = "Value")]
    pub value: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Playfield {
    #[serde(rename = "DrawObjects")]
    pub draw_objects: bool,
    #[serde(rename = "DrawCursors")]
    pub draw_cursors: bool,
    #[serde(rename = "Scale")]
    pub scale: i64,
    #[serde(rename = "OsuShift")]
    pub osu_shift: bool,
    #[serde(rename = "ShiftY")]
    pub shift_y: i64,
    #[serde(rename = "ShiftX")]
    pub shift_x: i64,
    #[serde(rename = "ScaleStoryboardWithPlayfield")]
    pub scale_storyboard_with_playfield: bool,
    #[serde(rename = "LeadInTime")]
    pub lead_in_time: i64,
    #[serde(rename = "LeadInHold")]
    pub lead_in_hold: i64,
    #[serde(rename = "FadeOutTime")]
    pub fade_out_time: i64,
    #[serde(rename = "SeizureWarning")]
    pub seizure_warning: SeizureWarning,
    #[serde(rename = "Background")]
    pub background: Background,
    #[serde(rename = "Logo")]
    pub logo: Logo,
    #[serde(rename = "Bloom")]
    pub bloom: Bloom,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeizureWarning {
    #[serde(rename = "Enabled")]
    pub enabled: bool,
    #[serde(rename = "Duration")]
    pub duration: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Background {
    #[serde(rename = "LoadStoryboards")]
    pub load_storyboards: bool,
    #[serde(rename = "LoadVideos")]
    pub load_videos: bool,
    #[serde(rename = "FlashToTheBeat")]
    pub flash_to_the_beat: bool,
    #[serde(rename = "Dim")]
    pub dim: Dim,
    #[serde(rename = "Parallax")]
    pub parallax: Parallax,
    #[serde(rename = "Blur")]
    pub blur: Blur,
    #[serde(rename = "Triangles")]
    pub triangles: Triangles,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dim {
    #[serde(rename = "Intro")]
    pub intro: i64,
    #[serde(rename = "Normal")]
    pub normal: f64,
    #[serde(rename = "Breaks")]
    pub breaks: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parallax {
    #[serde(rename = "Amount")]
    pub amount: i64,
    #[serde(rename = "Speed")]
    pub speed: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Blur {
    #[serde(rename = "Enabled")]
    pub enabled: bool,
    #[serde(rename = "Values")]
    pub values: Values,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Values {
    #[serde(rename = "Intro")]
    pub intro: i64,
    #[serde(rename = "Normal")]
    pub normal: f64,
    #[serde(rename = "Breaks")]
    pub breaks: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Triangles {
    #[serde(rename = "Enabled")]
    pub enabled: bool,
    #[serde(rename = "Shadowed")]
    pub shadowed: bool,
    #[serde(rename = "DrawOverBlur")]
    pub draw_over_blur: bool,
    #[serde(rename = "ParallaxMultiplier")]
    pub parallax_multiplier: i64,
    #[serde(rename = "Density")]
    pub density: i64,
    #[serde(rename = "Scale")]
    pub scale: i64,
    #[serde(rename = "Speed")]
    pub speed: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Logo {
    #[serde(rename = "DrawSpectrum")]
    pub draw_spectrum: bool,
    #[serde(rename = "Dim")]
    pub dim: Dim2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dim2 {
    #[serde(rename = "Intro")]
    pub intro: i64,
    #[serde(rename = "Normal")]
    pub normal: i64,
    #[serde(rename = "Breaks")]
    pub breaks: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bloom {
    #[serde(rename = "Enabled")]
    pub enabled: bool,
    #[serde(rename = "BloomToTheBeat")]
    pub bloom_to_the_beat: bool,
    #[serde(rename = "BloomBeatAddition")]
    pub bloom_beat_addition: f64,
    #[serde(rename = "Threshold")]
    pub threshold: i64,
    #[serde(rename = "Blur")]
    pub blur: f64,
    #[serde(rename = "Power")]
    pub power: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CursorDance {
    #[serde(rename = "Movers")]
    pub movers: Vec<Mover>,
    #[serde(rename = "Spinners")]
    pub spinners: Vec<Spinner>,
    #[serde(rename = "ComboTag")]
    pub combo_tag: bool,
    #[serde(rename = "Battle")]
    pub battle: bool,
    #[serde(rename = "DoSpinnersTogether")]
    pub do_spinners_together: bool,
    #[serde(rename = "TAGSliderDance")]
    pub tagslider_dance: bool,
    #[serde(rename = "MoverSettings")]
    pub mover_settings: MoverSettings,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mover {
    #[serde(rename = "Mover")]
    pub mover: String,
    #[serde(rename = "SliderDance")]
    pub slider_dance: bool,
    #[serde(rename = "RandomSliderDance")]
    pub random_slider_dance: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Spinner {
    #[serde(rename = "Mover")]
    pub mover: String,
    #[serde(rename = "Radius")]
    pub radius: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoverSettings {
    #[serde(rename = "Bezier")]
    pub bezier: Vec<Bezier>,
    #[serde(rename = "Flower")]
    pub flower: Vec<Flower>,
    #[serde(rename = "HalfCircle")]
    pub half_circle: Vec<HalfCircle>,
    #[serde(rename = "Spline")]
    pub spline: Vec<Spline>,
    #[serde(rename = "Momentum")]
    pub momentum: Vec<Momentum>,
    #[serde(rename = "ExGon")]
    pub ex_gon: Vec<ExGon>,
    #[serde(rename = "Linear")]
    pub linear: Vec<Linear>,
    #[serde(rename = "Pippi")]
    pub pippi: Vec<Pippi>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bezier {
    #[serde(rename = "Aggressiveness")]
    pub aggressiveness: i64,
    #[serde(rename = "SliderAggressiveness")]
    pub slider_aggressiveness: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Flower {
    #[serde(rename = "AngleOffset")]
    pub angle_offset: i64,
    #[serde(rename = "DistanceMult")]
    pub distance_mult: f64,
    #[serde(rename = "StreamAngleOffset")]
    pub stream_angle_offset: i64,
    #[serde(rename = "LongJump")]
    pub long_jump: i64,
    #[serde(rename = "LongJumpMult")]
    pub long_jump_mult: f64,
    #[serde(rename = "LongJumpOnEqualPos")]
    pub long_jump_on_equal_pos: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HalfCircle {
    #[serde(rename = "RadiusMultiplier")]
    pub radius_multiplier: i64,
    #[serde(rename = "StreamTrigger")]
    pub stream_trigger: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Spline {
    #[serde(rename = "RotationalForce")]
    pub rotational_force: bool,
    #[serde(rename = "StreamHalfCircle")]
    pub stream_half_circle: bool,
    #[serde(rename = "StreamWobble")]
    pub stream_wobble: bool,
    #[serde(rename = "WobbleScale")]
    pub wobble_scale: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Momentum {
    #[serde(rename = "SkipStackAngles")]
    pub skip_stack_angles: bool,
    #[serde(rename = "StreamRestrict")]
    pub stream_restrict: bool,
    #[serde(rename = "DurationMult")]
    pub duration_mult: i64,
    #[serde(rename = "DurationTrigger")]
    pub duration_trigger: i64,
    #[serde(rename = "StreamMult")]
    pub stream_mult: f64,
    #[serde(rename = "RestrictAngle")]
    pub restrict_angle: i64,
    #[serde(rename = "RestrictArea")]
    pub restrict_area: i64,
    #[serde(rename = "RestrictInvert")]
    pub restrict_invert: bool,
    #[serde(rename = "DistanceMult")]
    pub distance_mult: f64,
    #[serde(rename = "DistanceMultOut")]
    pub distance_mult_out: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExGon {
    #[serde(rename = "Delay")]
    pub delay: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Linear {
    #[serde(rename = "WaitForPreempt")]
    pub wait_for_preempt: bool,
    #[serde(rename = "ReactionTime")]
    pub reaction_time: i64,
    #[serde(rename = "ChoppyLongObjects")]
    pub choppy_long_objects: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pippi {
    #[serde(rename = "RotationSpeed")]
    pub rotation_speed: f64,
    #[serde(rename = "RadiusMultiplier")]
    pub radius_multiplier: f64,
    #[serde(rename = "SpinnerRadius")]
    pub spinner_radius: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Knockout {
    #[serde(rename = "Mode")]
    pub mode: i64,
    #[serde(rename = "ExcludeMods")]
    pub exclude_mods: String,
    #[serde(rename = "HideMods")]
    pub hide_mods: String,
    #[serde(rename = "MaxPlayers")]
    pub max_players: i64,
    #[serde(rename = "BubbleMinimumCombo")]
    pub bubble_minimum_combo: i64,
    #[serde(rename = "RevivePlayersAtEnd")]
    pub revive_players_at_end: bool,
    #[serde(rename = "LiveSort")]
    pub live_sort: bool,
    #[serde(rename = "SortBy")]
    pub sort_by: String,
    #[serde(rename = "HideOverlayOnBreaks")]
    pub hide_overlay_on_breaks: bool,
    #[serde(rename = "MinCursorSize")]
    pub min_cursor_size: i64,
    #[serde(rename = "MaxCursorSize")]
    pub max_cursor_size: i64,
    #[serde(rename = "AddDanser")]
    pub add_danser: bool,
    #[serde(rename = "DanserName")]
    pub danser_name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Recording {
    #[serde(rename = "FrameWidth")]
    pub frame_width: i64,
    #[serde(rename = "FrameHeight")]
    pub frame_height: i64,
    #[serde(rename = "FPS")]
    pub fps: i64,
    #[serde(rename = "EncodingFPSCap")]
    pub encoding_fpscap: i64,
    #[serde(rename = "Encoder")]
    pub encoder: String,
    #[serde(rename = "EncoderOptions")]
    pub encoder_options: String,
    #[serde(rename = "Profile")]
    pub profile: String,
    #[serde(rename = "Preset")]
    pub preset: String,
    #[serde(rename = "PixelFormat")]
    pub pixel_format: String,
    #[serde(rename = "Filters")]
    pub filters: String,
    #[serde(rename = "AudioCodec")]
    pub audio_codec: String,
    #[serde(rename = "AudioOptions")]
    pub audio_options: String,
    #[serde(rename = "AudioFilters")]
    pub audio_filters: String,
    #[serde(rename = "OutputDir")]
    pub output_dir: String,
    #[serde(rename = "Container")]
    pub container: String,
    #[serde(rename = "ShowFFmpegLogs")]
    pub show_ffmpeg_logs: bool,
    #[serde(rename = "MotionBlur")]
    pub motion_blur: MotionBlur,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MotionBlur {
    #[serde(rename = "Enabled")]
    pub enabled: bool,
    #[serde(rename = "OversampleMultiplier")]
    pub oversample_multiplier: i64,
    #[serde(rename = "BlendFrames")]
    pub blend_frames: i64,
    #[serde(rename = "BlendWeights")]
    pub blend_weights: BlendWeights,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlendWeights {
    #[serde(rename = "UseManualWeights")]
    pub use_manual_weights: bool,
    #[serde(rename = "ManualWeights")]
    pub manual_weights: String,
    #[serde(rename = "AutoWeightsID")]
    pub auto_weights_id: i64,
    #[serde(rename = "GaussWeightsMult")]
    pub gauss_weights_mult: f64,
}
