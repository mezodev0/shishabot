use std::{
    collections::VecDeque,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    path::PathBuf,
};

use osu_db::Replay;
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    Mutex,
};
use twilight_model::id::{
    marker::{ChannelMarker, UserMarker},
    Id,
};

// TODO: use SlimReplay for smaller size and cheaper cloning
#[derive(Clone)]
pub struct ReplayData {
    pub input_channel: Id<ChannelMarker>,
    pub output_channel: Id<ChannelMarker>,
    pub path: PathBuf,
    pub replay: Replay,
    pub time_points: TimePoints,
    pub user: Id<UserMarker>,
}

#[derive(Copy, Clone)]
pub struct TimePoints {
    pub start: Option<u16>,
    pub end: Option<u16>,
}

impl TimePoints {
    pub fn parse_single(s: &str) -> Result<u32, &'static str> {
        let mut iter = s.split(':').map(str::parse);

        match (iter.next(), iter.next()) {
            (Some(Ok(minutes)), Some(Ok(seconds @ 0..=59))) => Ok(minutes * 60 + seconds),
            (Some(Ok(_)), Some(Ok(_))) => Err("Seconds must be between 0 and 60!"),
            (Some(Ok(seconds)), None) => Ok(seconds),
            _ => Err("A value you supplied is not a number!"),
        }
    }
}

pub struct ReplayQueue {
    pub queue: Mutex<VecDeque<ReplayData>>,
    pub status: Mutex<ReplayStatus>,
    tx: UnboundedSender<()>,
    rx: Mutex<UnboundedReceiver<()>>,
}

#[derive(Copy, Clone, Debug)]
pub enum ReplayStatus {
    Waiting,
    Downloading,
    Processing,
    Uploading,
}

impl ReplayQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn push(&self, data: ReplayData) {
        self.queue.lock().await.push_back(data);
        let _ = self.tx.send(());
    }

    pub async fn pop(&self) -> ReplayData {
        self.queue.lock().await.pop_front().unwrap()
    }

    pub async fn peek(&self) -> ReplayData {
        let mut guard = self.rx.lock().await;
        let _ = guard.recv().await;

        self.queue.lock().await.front().unwrap().to_owned()
    }

    pub async fn set_status(&self, status: ReplayStatus) {
        *self.status.lock().await = status;
    }

    pub async fn reset_peek(&self) {
        *self.status.lock().await = ReplayStatus::Waiting;
        let _ = self.pop().await;
    }
}

impl Default for ReplayQueue {
    #[inline]
    fn default() -> Self {
        let (tx, rx) = unbounded_channel();

        Self {
            queue: Mutex::new(VecDeque::new()),
            tx,
            rx: Mutex::new(rx),
            status: Mutex::new(ReplayStatus::Waiting),
        }
    }
}

impl Display for ReplayStatus {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        <Self as Debug>::fmt(self, f)
    }
}
