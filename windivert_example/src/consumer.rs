use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use anyhow::{Ok, Result};
use log::*;
use tokio::{sync::RwLock, task::{self, JoinHandle}};

use crate::wrapper::WindivertWrapper;

pub struct Consumer {
    handle: Option<JoinHandle<Result<()>>>,
    windivert: Arc<RwLock<WindivertWrapper>>,
    shutdown: Arc<AtomicBool>
}

impl Consumer {
    pub fn new() -> Self {
        let shutdown = Arc::new(AtomicBool::new(false));
        let ip_address = "127.0.0.1"; 
        let port = 443;
        let windivert = Arc::new(RwLock::new(WindivertWrapper::new(ip_address, port).unwrap()));

        Self {
            windivert,
            handle: None,
            shutdown
        }
    }

    pub async fn start(&mut self) -> Result<()> {
   
        let shutdown = self.shutdown.clone();
        let windivert = self.windivert.clone();
        let handle = task::spawn(Self::consume(windivert, shutdown));

        self.handle = Some(handle);

        Ok(())
    }

    async fn consume(windivert: Arc<RwLock<WindivertWrapper>>, shutdown: Arc<AtomicBool>) -> Result<()> {

        debug!("windivert.start");
        windivert.write().await.start().await?;

        debug!("recv loop");
        while !shutdown.load(Ordering::Relaxed) {
            debug!("windivert.recv");
            match windivert.write().await.recv().await {
                Some(data) => {
                    info!("Packet length: {}", data.len());
                },
                None => todo!(),
            }
        }

        debug!("end recv loop");
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        debug!("stop");
        self.shutdown.store(true, Ordering::Relaxed);
        debug!("self.windivert.write");
        self.windivert.write().await.stop().await?;

        debug!("self.handle.take");
        if let Some(handle) = self.handle.take() {
            if let Err(err) = handle.await {
                error!("Error stopping thread: {:?}", err);
            }
        }

        Ok(())
    }
}
