use std::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    path::PathBuf,
};

use osu_db::Replay;
use twilight_model::id::{
    marker::{ChannelMarker, UserMarker},
    Id,
};

#[derive(Clone)]
pub struct ReplayData {
    pub input_channel: Id<ChannelMarker>,
    pub output_channel: Id<ChannelMarker>,
    pub path: PathBuf,
    pub replay: ReplaySlim,
    pub time_points: TimePoints,
    pub user: Id<UserMarker>,
}

#[derive(Copy, Clone)]
pub struct TimePoints {
    pub start: Option<u16>,
    pub end: Option<u16>,
}

#[derive(Copy, Clone, Debug)]
pub enum ReplayStatus {
    Waiting,
    Downloading,
    Rendering(u8),
    Encoding(u8),
    Uploading,
}

impl Display for ReplayStatus {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Waiting => f.write_str("Waiting"),
            Self::Downloading => f.write_str("Downloading"),
            Self::Rendering(progress) => write!(f, "Rendering: {progress}%"),
            Self::Encoding(progress) => write!(f, "Encoding: {progress}%"),
            Self::Uploading => f.write_str("Uploading"),
        }
    }
}

#[derive(Clone)]
pub struct ReplaySlim {
    pub beatmap_hash: Option<String>,
    pub count_300: u16,
    pub count_100: u16,
    pub count_50: u16,
    pub count_geki: u16,
    pub count_katsu: u16,
    pub count_miss: u16,
    pub max_combo: u16,
    pub mods: u32,
    pub player_name: Option<String>,
}

impl ReplaySlim {
    pub fn total_hits(&self) -> u16 {
        self.count_300 + self.count_100 + self.count_50 + self.count_miss
    }

    pub fn accuracy(&self) -> f32 {
        let numerator = (self.count_50 as u32 * 50
            + self.count_100 as u32 * 100
            + self.count_300 as u32 * 300) as f32;

        let denominator = self.total_hits() as f32 * 300.0;

        (10_000.0 * numerator / denominator).round() / 100.0
    }
}

impl From<Replay> for ReplaySlim {
    #[inline]
    fn from(replay: Replay) -> Self {
        Self {
            beatmap_hash: replay.beatmap_hash,
            count_300: replay.count_300,
            count_100: replay.count_100,
            count_50: replay.count_50,
            count_geki: replay.count_geki,
            count_katsu: replay.count_katsu,
            count_miss: replay.count_miss,
            max_combo: replay.max_combo,
            mods: replay.mods.bits(),
            player_name: replay.player_name,
        }
    }
}
