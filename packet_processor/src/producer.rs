use std::{sync::mpsc, thread, time::Duration};

use bincode::{config::Configuration, Decode};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::models::Packet;

pub struct Producer {
    handle: Option<thread::JoinHandle<Result<(), anyhow::Error>>>
}

impl Producer {
    pub fn new() -> Self {
        Self {
            handle: None
        }
    }

    pub fn start(&mut self, port: u16) -> UnboundedReceiver<Vec<u8>> {
        let config = bincode::config::standard();

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();

        let handle = thread::spawn(move || {

            let packet = Packet::NewPlayer { 
                id: 1,
                name: "".to_string()
            };
            let data = bincode::encode_to_vec(packet, config)?;

            tx.send(data)?;
            anyhow::Ok(())
        });

        self.handle = Some(handle);

        rx
    }

    pub fn stop(&mut self) {

    }
}