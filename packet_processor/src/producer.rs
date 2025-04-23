use std::{sync::{atomic::{AtomicBool, Ordering}, Arc}, thread};

use log::error;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::source::Source;

pub struct Producer {
    source: Option<Source>,
    handle: Option<thread::JoinHandle<Result<(), anyhow::Error>>>,
    rx: Option<UnboundedReceiver<Vec<u8>>>,
    stop_flag: Arc<AtomicBool>,
}

impl Producer {
    pub fn new(source: Source) -> Self {
        Self {
            source: Some(source),
            handle: None,
            rx: None,
            stop_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&mut self, _port: u16) {

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
        let mut source = self.source.take().expect("Source not set");
        let stop_flag = self.stop_flag.clone();

        let handle = thread::spawn(move || {

            loop {
                if stop_flag.load(Ordering::Relaxed) {
                    break;
                }

                source.run(tx.clone());
                // tx.send(data)?;
            }
           
            anyhow::Ok(())
        });

        self.handle = Some(handle);
        self.rx = Some(rx);
    }

    pub async fn recv(&mut self) -> Option<Vec<u8>> {

        if let Some(rx) = self.rx.as_mut() {
            let data = rx.recv().await;

            if data.is_none() {
                self.rx.take();
            }

            return data
        }

        None
    }

    pub fn is_running(&self) -> bool {
        if let Some(handle) = &self.handle {
            return !handle.is_finished()
        }

        return false
    }

    pub fn stop(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        self.rx = None;

        if let Some(handle) = self.handle.take() {
            match handle.join() {
                Ok(result) => {
                    if let Err(err) = result {
                        error!("Producer error: {:?}", err);
                    }
                }
                Err(err) => {
                    error!("Failed to join thread: {:?}", err);
                }
            }
        }
    }
}