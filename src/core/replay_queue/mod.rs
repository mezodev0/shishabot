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
        trace!("Locking channel receiver...");
        let mut guard = self.rx.lock().await;
        trace!("Locked receiver, awaiting entry...");
        let _ = guard.recv().await;
        trace!("Received entry, locking queue...");
        let queue_guard = self.queue.lock().await;
        trace!("Locked queue");

        queue_guard.front().unwrap().to_owned()
    }

    pub async fn set_status(&self, status: ReplayStatus) {
        trace!("Updating progress status to {status:?}...");
        *self.status.lock().await = status;
        trace!("Updated progress status");
    }

    pub async fn reset_peek(&self) {
        trace!("Resetting peek...");
        *self.status.lock().await = ReplayStatus::Waiting;
        trace!("Peek reset, popping queue...");
        let _ = self.pop().await;
        trace!("Popped queue");
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
