use std::collections::VecDeque;

use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    Mutex,
};

pub use self::data::*;

mod data;
mod process;

pub struct ReplayQueue {
    pub queue: Mutex<VecDeque<ReplayData>>,
    pub status: Mutex<ReplayStatus>,
    tx: UnboundedSender<()>,
    rx: Mutex<UnboundedReceiver<()>>,
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
