use std::fmt::Write;

use command_macros::pagination;
use time::OffsetDateTime;
use twilight_model::channel::embed::Embed;

use crate::util::builder::{AuthorBuilder, EmbedBuilder, FooterBuilder};

use super::Pages;

#[pagination(per_page = 15, entries = "cmd_counts")]
pub struct CommandCountPagination {
    booted_up: OffsetDateTime,
    cmd_counts: Vec<(String, u32)>,
}

impl CommandCountPagination {
    pub fn build_page(&mut self, pages: &Pages) -> Embed {
        let sub_list: Vec<(&String, u32)> = self
            .cmd_counts
            .iter()
            .skip(pages.index)
            .take(pages.per_page)
            .map(|(name, amount)| (name, *amount))
            .collect();

        let len = sub_list
            .iter()
            .fold(0, |max, (name, _)| max.max(name.chars().count()));

        let mut description = String::with_capacity(256);
        description.push_str("```\n");

        for ((name, amount), i) in sub_list.into_iter().zip(pages.index + 1..) {
            let _ = writeln!(description, "{i:>2} # {name:<len$} => {amount}");
        }

        description.push_str("```");

        let page = pages.curr_page();
        let pages = pages.last_page();

        let footer_text = format!("Page {page}/{pages}");

        EmbedBuilder::new()
            .description(description)
            .footer(FooterBuilder::new(footer_text))
            .author(AuthorBuilder::new("Most popular commands:"))
            .build()
    }
}
