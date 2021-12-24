#[derive(Deserialize)]
struct Settings {
    #[serde(rename = "Skin")]
    skin: Skin,
    #[serde(rename = "Gameplay")]
    gameplay: Gameplay,
    #[serde(rename = "Cursor")]
    cursor: Cursor,
    #[serde(rename = "Playfield")]
    playfield: Playfield,
}

#[derive(Deserialize)]
struct Skin {
    #[serde(rename = "CurrentSkin")]
    currentSkin: CurrentSkin,
    #[serde(rename = "Cursor")]
    cursor: SkinCursor,
}

#[derive(Deserialize)]
struct CurrentSkin {
    #[serde(rename = "CurrentSkin")]
    currentSkin: String,
}

#[derive(Deserialize)]
struct SkinCursor {
    #[serde(rename = "Scale")]
    scale: f64,
}

#[derive(Deserialize)]
struct Gameplay {
    #[serde(rename = "HitErrorMeter")]
    hitErrorMeter: HitErrorMeter,
    #[serde(rename = "AimErrorMeter")]
    aimErrorMeter: AimErrorMeter,
    #[serde(rename = "PPCounter")]
    ppCounter: PPCounter,
}

#[derive(Deserialize)]
struct HitErrorMeter {
    #[serde(rename = "Show")]
    show: bool,
    #[serde(rename = "UnstableRateDecimals")]
    unstableRateDecimals: u64,
}

#[derive(Deserialize)]
struct AimErrorMeter {
    #[serde(rename = "Show")]
    show: bool,
    #[serde(rename = "UnstableRateDecimals")]
    unstableRateDecimals: u64,
}

#[derive(Deserialize)]
struct PPCounter {
    #[serde(rename = "Show")]
    show: bool,
    #[serde(rename = "Decimals")]
    decimals: u64,
}

#[derive(Deserialize)]
struct Cursor {
    #[serde(rename = "CursorRipples")]
    cursorRipples: bool,
}

#[derive(Deserialize)]
struct Playfield {
    #[serde(rename = "Background")]
    background: Background,
}

#[derive(Deserialize)]
struct Background {
    #[serde(rename = "LoadStoryboards")]
    loadStoryboards: bool,
    #[serde(rename = "LoadVideos")]
    loadVideos: bool,
    #[serde(rename = "Dim")]
    dim: Dim,
}

#[derive(Deserialize)]
struct Dim {
    #[serde(rename = "Normal")]
    normal: f64,
}
