mod utility;

use twilight_model::channel::embed::{Embed, EmbedField};

pub use self::utility::*;

type EmbedFields = Vec<EmbedField>;

pub trait EmbedData {
    fn build(self) -> Embed;
}

impl EmbedData for Embed {
    fn build(self) -> Embed {
        self
    }
}

pub fn attachment(filename: impl AsRef<str>) -> String {
    #[cfg(debug_assert)]
    match filename.rfind('.') {
        Some(idx) => {
            if filename.get(idx + 1..).map(str::is_empty).is_none() {
                panic!("expected non-empty extension for attachment");
            }
        }
        None => panic!("expected extension for attachment"),
    }

    format!("attachment://{}", filename.as_ref())
}
