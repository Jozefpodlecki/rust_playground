use std::{error::Error, net::Ipv4Addr, sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex, RwLock}, thread::{self, JoinHandle}, vec};

use anyhow::{Result};
use windivert::{layer::NetworkLayer, packet::WinDivertPacket, prelude::WinDivertFlags, CloseAction, WinDivert};
use tokio::{select, signal::windows::{self, ctrl_c}, task};

pub struct WindivertWrapper {
    windivert: Arc<RwLock<WinDivert<NetworkLayer>>>,
    handle: Option<JoinHandle<()>>
}

impl WindivertWrapper {
    pub fn new() -> Result<Self> {
        let ip = "127.0.0.1"; 
        let port = 6041;
    
        let filter = format!("ip.SrcAddr == {ip} and tcp.SrcPort == {port}");
        let flags = WinDivertFlags::new().set_recv_only().set_sniff();
        let windivert = WinDivert::network(&filter, 0, flags)?;
        let windivert = Arc::new(RwLock::new(windivert));

        Ok(Self {
            windivert,
            handle: None
        })
    }
    
    pub fn start(&mut self) {
      
        let windivert = self.windivert.clone();

        let handle = thread::spawn(move || {
            let mut buffer = vec![0u8; 65535];

            while let Ok(windivert) = windivert.read() {
                match windivert.recv(Some(&mut buffer)) {
                    Ok(packet) => {

                    },
                    Err(err) => {

                    },
                }
            }
        });

        self.handle = Some(handle);
    }

    pub fn wait(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.join();
        }
    }

    pub fn stop(&mut self) {
        self.windivert.write().unwrap().close(CloseAction::Nothing);
    }
}