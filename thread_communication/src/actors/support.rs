use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, sleep, JoinHandle};
use anyhow::*;
use chrono::{DateTime, Duration, Utc};
use log::{debug, warn};
use multiqueue::BroadcastSender;

use crate::models::*;

use rand::Rng;

pub struct SupportThread {
    handle: Option<JoinHandle<Result<()>>>,
    close_flag: Arc<AtomicBool>
}

impl SupportThread {
    pub fn new() -> Self {
        Self {
            handle: None,
            close_flag: Arc::new(AtomicBool::new(false))
        }
    }

    pub fn start(&mut self, tx: BroadcastSender<Message>, rx: BroadcastSender<Message>) {
        let start_time = Utc::now();
        let close_flag = self.close_flag.clone();

        let handle = thread::spawn(move || {
          

            loop {
                if close_flag.load(Ordering::Relaxed) {
                    debug!("Stopping");
                    break;
                }


            }

            Ok(())
        });

        self.handle = Some(handle);
    }

    pub fn wait(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap().unwrap();
        }
    }

    pub fn stop(&mut self) {
        self.close_flag.store(true, Ordering::Relaxed);
        self.wait();
    }
}