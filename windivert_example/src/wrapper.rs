use std::sync::{atomic::{AtomicBool, Ordering}, Arc};

use anyhow::{Ok, Result};
use log::*;
use tokio::{sync::mpsc::{UnboundedReceiver, UnboundedSender}, task::{self, JoinHandle}};
use windivert::{layer::NetworkLayer, prelude::WinDivertFlags, CloseAction, WinDivert};

pub struct WindivertWrapper {
    ip_address: String,
    port: u16,
    handle: Option<JoinHandle<Result<()>>>,
    tx: UnboundedSender<Vec<u8>>,
    rx: UnboundedReceiver<Vec<u8>>,
    shutdown: Arc<AtomicBool>, 
}

impl WindivertWrapper {
    pub fn new(ip_address: &str, port: u16) -> Result<Self> {
        // let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let shutdown = Arc::new(AtomicBool::new(false)); 

        Ok(Self {
            ip_address: ip_address.into(),
            rx,
            tx,
            port,
            handle: None,
            shutdown
        })
    }
    
    pub async fn start(&mut self) -> Result<()> {
        debug!("start");
        let tx = self.tx.clone();
        let shutdown = self.shutdown.clone();

        let filter = format!("tcp.SrcPort == {}", self.port);
        let flags = WinDivertFlags::new().set_recv_only().set_sniff();
        let windivert = WinDivert::network(&filter, 0, flags)?;
        let handle = task::spawn(Self::listen(windivert, tx, shutdown));
        self.handle = Some(handle);

        Ok(())
    }

    async fn listen(
        mut windivert: WinDivert<NetworkLayer>,
        tx: UnboundedSender<Vec<u8>>,
        shutdown: Arc<AtomicBool>) -> Result<()> {
        let mut buffer = vec![0u8; 65535];

        debug!("listen loop");
        while !shutdown.load(Ordering::Relaxed) {
            match windivert.recv(Some(&mut buffer)) {
                std::result::Result::Ok(packet) => {
                    let _ = tx.send(packet.data.to_vec());
                    task::yield_now().await;
                }
                Err(err) => {
                    error!("{:?}", err);
                }
            }
        }

        windivert.close(CloseAction::Nothing)?;
        
        Ok(())
    }

    pub async fn recv(&mut self) -> Option<Vec<u8>> {
        self.rx.recv().await
    }

    pub async fn stop(&mut self) -> Result<()> {
        debug!("stop");
        self.shutdown.store(true, Ordering::Relaxed);

        debug!("handle.take");
        if let Some(handle) = self.handle.take() {
            if let Err(err) = handle.await {
                error!("Error stopping thread: {:?}", err);
            }
        }

        Ok(())
    }
}