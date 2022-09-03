use std::fmt::Write;

use command_macros::pagination;
use twilight_model::channel::embed::Embed;

use crate::util::builder::{EmbedBuilder, FooterBuilder};

use super::Pages;

#[pagination(per_page = 20, entries = "skins")]
pub struct SkinListPagination {
    skins: Vec<String>,
}

impl SkinListPagination {
    pub fn build_page(&mut self, pages: &Pages) -> Embed {
        let mut description = String::with_capacity(256);

        let skins = self
            .skins
            .iter()
            .skip(pages.index)
            .take(pages.per_page)
            .zip(pages.index + 1..);

        for (skin, idx) in skins {
            let _ = writeln!(description, "{idx}) {skin}");
        }

        let page = pages.curr_page();
        let pages = pages.last_page();

        let footer_text = format!("Page {page}/{pages}");

        EmbedBuilder::new()
            .description(description)
            .footer(FooterBuilder::new(footer_text))
            .build()
    }
}
