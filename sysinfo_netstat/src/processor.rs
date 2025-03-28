use std::{sync::{atomic::{AtomicBool, Ordering}, mpsc::{Receiver, Sender}, Arc}, thread::{sleep, JoinHandle}, time::Duration};

use anyhow::*;
use crate::models::{Action, Message};

pub struct Processor {
    handle: Option<JoinHandle<Result<()>>>,
    close_flag: Arc<AtomicBool>,
}

impl Processor {
    pub fn new() -> Self {
        Self {
            handle: None,
            close_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&mut self) -> Receiver<Action> {
        let (tx, rx) = std::sync::mpsc::channel::<Action>();

        let close_flag = self.close_flag.clone();
        let handle = std::thread::spawn(move || Self::generate(close_flag, tx));

        self.handle = Some(handle);
        
        rx
    }

    fn generate(close_flag: Arc<AtomicBool>,tx: Sender<Action>) -> Result<()> {
        let timeout = Duration::from_secs(15);

        while !close_flag.load(Ordering::Relaxed) {

            tx.send(Action::Task)?;

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
