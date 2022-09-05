use std::{ffi::OsString, fs};

use eyre::{Context as _, Result};

use crate::core::BotConfig;

/// Cache skin names to avoid IO interactions
#[derive(Default)]
pub struct SkinList {
    skins: Option<Vec<OsString>>,
}

impl SkinList {
    pub fn get(&mut self) -> Result<&[OsString]> {
        if let Some(ref skins) = self.skins {
            return Ok(skins);
        }

        let path = BotConfig::get().paths.skins();

        let mut skins = fs::read_dir(&path)
            .context("failed to read skins folder")?
            .map(|res| res.map(|entry| entry.file_name()))
            .collect::<Result<Vec<_>, _>>()
            .context("failed to read entry of skins folder")?;

        skins.sort_unstable_by_key(|name| name.to_ascii_lowercase());

        info!("Repopulated skin list cache");

        Ok(self.skins.insert(skins))
    }

    pub fn clear(&mut self) {
        self.skins = None;

        info!("Cleared skin list cache");
    }
}
