use std::fmt;

use time::OffsetDateTime;

pub struct SecToMinSecFormatter {
    secs: u32,
}

impl fmt::Display for SecToMinSecFormatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{:02}", self.secs / 60, self.secs % 60)
    }
}

/// Instead of writing the whole string like `how_long_ago_text`,
/// this just writes discord's syntax for dynamic timestamps and lets
/// discord handle the rest.
///
/// Note: Doesn't work in embed footers
pub fn how_long_ago_dynamic(date: &OffsetDateTime) -> HowLongAgoFormatterDynamic {
    HowLongAgoFormatterDynamic(date.unix_timestamp())
}

#[derive(Copy, Clone)]
pub struct HowLongAgoFormatterDynamic(i64);

impl fmt::Display for HowLongAgoFormatterDynamic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // https://discord.com/developers/docs/reference#message-formatting-timestamp-styles
        write!(f, "<t:{}:R>", self.0)
    }
}
