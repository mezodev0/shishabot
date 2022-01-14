use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Settings {
    pub skin: Skin,
    pub gameplay: Gameplay,
    pub cursor: Cursor,
    pub playfield: Playfield,
    pub audio: Audio,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Skin {
    pub current_skin: String,
    pub cursor: SkinCursor,
}

#[derive(Deserialize, Debug)]
pub struct SkinCursor {
    #[serde(rename = "Scale")]
    pub scale: f64,
}

#[derive(Deserialize, Debug)]
pub struct Gameplay {
    #[serde(rename = "HitErrorMeter")]
    pub hit_error_meter: ErrorMeter,
    #[serde(rename = "AimErrorMeter")]
    pub aim_error_meter: ErrorMeter,
    #[serde(rename = "PPCounter")]
    pub pp_counter: PPCounter,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ErrorMeter {
    pub show: bool,
    pub unstable_rate_decimals: u64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct PPCounter {
    pub show: bool,
    pub decimals: u64,
}

#[derive(Deserialize, Debug)]
pub struct Cursor {
    #[serde(rename = "CursorRipples")]
    pub cursor_ripples: bool,
}

#[derive(Deserialize, Debug)]
pub struct Playfield {
    #[serde(rename = "Background")]
    pub background: Background,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Background {
    pub load_storyboards: bool,
    pub load_videos: bool,
    pub dim: Dim,
}

#[derive(Deserialize, Debug)]
pub struct Dim {
    #[serde(rename = "Normal")]
    pub normal: f64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Audio {
    pub music_volume: f64,
    pub sample_volume: f64,
}
