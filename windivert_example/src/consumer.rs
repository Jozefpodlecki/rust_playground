use std::{fs::File, io::BufReader, path::Path, sync::{atomic::AtomicBool, Arc}, thread::JoinHandle};
use anyhow::{Ok, Result};
use log::*;
use tokio::{runtime::Runtime, sync::{mpsc::UnboundedReceiver, watch}, task::{self}};
use windivert::{prelude::WinDivertFlags, WinDivert};
use std::thread::spawn;

pub struct Consumer {
    handle: Option<JoinHandle<Result<()>>>,
    shutdown: Arc<AtomicBool>,
    shutdown_tx: Option<watch::Sender<()>>,
}

impl Consumer {
    pub fn new() -> Self {
        let shutdown = Arc::new(AtomicBool::new(false));

        Self {
            handle: None,
            shutdown,
            shutdown_tx: None
        }
    }

    pub async fn start(&mut self, ip_address: &str, port: u16) -> Result<UnboundedReceiver<Vec<u8>>> {
   
        let filter = format!("tcp.SrcPort == {}", port);
       
        let (shutdown_tx, mut shutdown_rx) = watch::channel(());
        self.shutdown_tx = Some(shutdown_tx);

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();

        let handle = spawn(move || {
            let runtime = Runtime::new()?;
            
            runtime.block_on(async {
                let flags = WinDivertFlags::new().set_recv_only().set_sniff();
                let windivert = Arc::new(WinDivert::network(&filter, 0, flags)?);
                
                loop {
                    let windivert = windivert.clone();
                    
                    let recv = task::spawn_blocking(move || {
                        let mut buffer = vec![0u8; 65535];
                        let result = windivert.recv(Some(&mut buffer)).unwrap();
                        let data = result.data.to_vec();
                        data
                    });
    
                    tokio::select! {
                        result = recv => {
                            use std::result::Result::Ok;
                            match result {
                                Ok(data) => {
                                    tx.send(data)?;
                                },
                                Err(err) => {
                                    info!("{err}");
                                },
                            }
                        }
                        _ = shutdown_rx.changed() => {
                            info!("shutdown");
                            break;
                        }
                    }
                }

                anyhow::Ok(())
            })?;

            anyhow::Ok(())
        });

        self.handle = Some(handle);

        Ok(rx)
    }


    pub fn stop(&mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }

        if let Some(handle) = self.handle.take() {
            if let Err(err) = handle.join() {
                error!("Error stopping thread: {:?}", err);
            }
        }

        Ok(())
    }
}
