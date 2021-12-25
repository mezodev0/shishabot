use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Settings {
    #[serde(rename = "Skin")]
    pub skin: Skin,
    #[serde(rename = "Gameplay")]
    pub gameplay: Gameplay,
    #[serde(rename = "Cursor")]
    pub cursor: Cursor,
    #[serde(rename = "Playfield")]
    pub playfield: Playfield,
    #[serde(rename = "Audio")]
    pub audio: Audio,
}

#[derive(Deserialize, Debug)]
pub struct Skin {
    #[serde(rename = "CurrentSkin")]
    pub currentSkin: String,
    #[serde(rename = "Cursor")]
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
    pub hitErrorMeter: HitErrorMeter,
    #[serde(rename = "AimErrorMeter")]
    pub aimErrorMeter: AimErrorMeter,
    #[serde(rename = "PPCounter")]
    pub ppCounter: PPCounter,
}

#[derive(Deserialize, Debug)]
pub struct HitErrorMeter {
    #[serde(rename = "Show")]
    pub show: bool,
    #[serde(rename = "UnstableRateDecimals")]
    pub unstableRateDecimals: u64,
}

#[derive(Deserialize, Debug)]
pub struct AimErrorMeter {
    #[serde(rename = "Show")]
    pub show: bool,
    #[serde(rename = "UnstableRateDecimals")]
    pub unstableRateDecimals: u64,
}

#[derive(Deserialize, Debug)]
pub struct PPCounter {
    #[serde(rename = "Show")]
    pub show: bool,
    #[serde(rename = "Decimals")]
    pub decimals: u64,
}

#[derive(Deserialize, Debug)]
pub struct Cursor {
    #[serde(rename = "CursorRipples")]
    pub cursorRipples: bool,
}

#[derive(Deserialize, Debug)]
pub struct Playfield {
    #[serde(rename = "Background")]
    pub background: Background,
}

#[derive(Deserialize, Debug)]
pub struct Background {
    #[serde(rename = "LoadStoryboards")]
    pub loadStoryboards: bool,
    #[serde(rename = "LoadVideos")]
    pub loadVideos: bool,
    #[serde(rename = "Dim")]
    pub dim: Dim,
}

#[derive(Deserialize, Debug)]
pub struct Dim {
    #[serde(rename = "Normal")]
    pub normal: f64,
}

#[derive(Deserialize, Debug)]
pub struct Audio {
    #[serde(rename = "MusicVolume")]
    pub musicVolume: f64,
    #[serde(rename = "SampleVolume")]
    pub sampleVolume: f64,
}
