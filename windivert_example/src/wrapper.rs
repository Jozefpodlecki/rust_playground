use std::{sync::{mpsc::{Receiver, Sender}, Arc, RwLock}, thread::{self, JoinHandle}};

use anyhow::Result;
use log::*;
use windivert::{layer::NetworkLayer, prelude::WinDivertFlags, CloseAction, WinDivert};

pub struct WindivertWrapper {
    windivert: Arc<RwLock<WinDivert<NetworkLayer>>>,
    handle: Option<JoinHandle<()>>,
    tx: Sender<Vec<u8>>,
    pub rx: Receiver<Vec<u8>>
}

impl WindivertWrapper {
    pub fn new(ip: &str, port: i32) -> Result<Self> {
    
        let filter = format!("ip.SrcAddr == {ip} and tcp.SrcPort == {port}");
        let flags = WinDivertFlags::new().set_recv_only().set_sniff();
        let windivert = WinDivert::network(&filter, 0, flags)?;
        let windivert = Arc::new(RwLock::new(windivert));
        let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();

        Ok(Self {
            windivert,
            rx,
            tx,
            handle: None
        })
    }
    
    pub fn start(&mut self) {
      
        let windivert = self.windivert.clone();
        let tx = self.tx.clone();

        let handle = thread::spawn(move || {
            let mut buffer = vec![0u8; 65535];

            while let Ok(windivert) = windivert.read() {
                match windivert.recv(Some(&mut buffer)) {
                    Ok(packet) => {
                        tx.send(packet.data.to_vec()).unwrap()
                    },
                    Err(err) => {
                        error!("{:?}", err);
                    },
                }
            }
        });

        self.handle = Some(handle);
    }

    pub fn wait(&mut self) -> Result<()> {
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }

        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        let mut windivert = self.windivert.write().unwrap();
        windivert.close(CloseAction::Nothing)?;

        Ok(())
    }
}