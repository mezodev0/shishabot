use time::OffsetDateTime;

pub struct BotStats {
    pub start_time: OffsetDateTime,
    // TODO: pub replays_rendered: IntCounter (shisha.mezo.xyz endpoint or maybe local counter)
}

impl BotStats {
    pub fn new() -> Self {
        Self {
            start_time: OffsetDateTime::now_utc(),
        }
    }
}
