use std::{sync::{atomic::{AtomicBool, Ordering}, mpsc::{Receiver, Sender}, Arc}, thread::{sleep, JoinHandle}, time::Duration};

use anyhow::*;
use crate::models::UpdateStatus;

pub struct UpdateChecker {
    handle: Option<JoinHandle<Result<()>>>,
    close_flag: Arc<AtomicBool>,
}

impl UpdateChecker {
    pub fn new() -> Self {
        Self {
            handle: None,
            close_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&mut self) -> UpdateStatus {
        let close_flag = self.close_flag.clone();
        let handle = std::thread::spawn(move || Self::check_periodically(close_flag));

        self.handle = Some(handle);

        UpdateStatus::Unknown
    }

    fn check_periodically(close_flag: Arc<AtomicBool>) -> Result<()> {
        let timeout = Duration::from_secs(15);

        while !close_flag.load(Ordering::Relaxed) {

            // tx.send(UpdateStatus::LatestVersion)?;

            sleep(timeout);
        }

        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        self.close_flag.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            handle
                .join()
                .map_err(|err| anyhow::anyhow!("{:?}", err))??;
        }

        Ok(())
    }
}
