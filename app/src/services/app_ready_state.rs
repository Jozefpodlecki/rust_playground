use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Notify;

pub struct AppReadyState {
    ready: AtomicBool,
    notify: Notify,
}

impl AppReadyState {
    pub fn new() -> Self {
        Self {
            ready: AtomicBool::new(false),
            notify: Notify::new(),
        }
    }

    pub fn mark_ready(&self) {
        self.ready.store(true, Ordering::Release);
        self.notify.notify_waiters();
    }

    pub async fn wait_for_ready(&self) {
        while !self.ready.load(Ordering::Acquire) {
            self.notify.notified().await;
        }
    }
}
