use std::{fs::File, io::BufReader, net::{Ipv4Addr, Ipv6Addr}, path::Path, sync::{atomic::AtomicBool, Arc}, thread::JoinHandle};
use anyhow::{Ok, Result};
use etherparse::{NetHeaders, PacketHeaders, TransportHeader};
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

    pub async fn start(&mut self, filter: String) -> Result<UnboundedReceiver<Vec<u8>>> {
   
        // let filter = format!("tcp.DstPort == {}", port);
        // let filter = format!("udp.SrcPort == {}", port);
       
        let (shutdown_tx, mut shutdown_rx) = watch::channel(());
        self.shutdown_tx = Some(shutdown_tx);

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();

        let flags = WinDivertFlags::new().set_recv_only().set_sniff();
        // WinDivert::flow(filter, priority, flags)

        let windivert = WinDivert::network(&filter, 0, flags)?;
        // let windivert = WinDivert::flow(&filter, 0, flags)?;

        let handle = spawn(move || {
            let runtime = Runtime::new()?;
            
            runtime.block_on(async {
              

                let windivert = Arc::new(windivert);
                
                loop {
                    let windivert = windivert.clone();
                    
                    let recv = task::spawn_blocking(move || {
                        let mut buffer = vec![0u8; 65535];
                        let result = windivert.recv(Some(&mut buffer)).unwrap();
                        let data = result.data.to_vec();
                        let headers = PacketHeaders::from_ip_slice(&data).unwrap();
                        // let test: etherparse::NetHeaders = headers.net.unwrap();
                        
                        let ip_summary = match headers.net {
                            Some(NetHeaders::Ipv4(ipv4, _)) => {
                                format!("{} -> {}", Ipv4Addr::from(ipv4.source), Ipv4Addr::from(ipv4.destination))
                            }
                            Some(NetHeaders::Ipv6(ipv6, _)) => {
                                format!("{} -> {}", Ipv6Addr::from(ipv6.source), Ipv6Addr::from(ipv6.destination))
                            }
                            _ => "Unknown IP".to_string(),
                        };
                        
                        let port_summary = match headers.transport {
                            Some(TransportHeader::Tcp(tcp)) => {
                                if tcp.source_port == 6040 || tcp.destination_port == 6040 {
                                    format!("TCP {} -> {}", tcp.source_port, tcp.destination_port)
                                } else {
                                    "".to_string() // Ignore if port is not 6040
                                }
                            }
                            Some(TransportHeader::Udp(udp)) => {
                                if udp.source_port == 6040 || udp.destination_port == 6040 {
                                    format!("UDP {} -> {}", udp.source_port, udp.destination_port)
                                } else {
                                    "".to_string() // Ignore if port is not 6040
                                }
                            }
                            _ => "".to_string(), // Ignore non-TCP/UDP packets
                        };
                        
                        if !port_summary.is_empty() {
                            println!("{} {}", ip_summary, port_summary);
                            return Some(data)
                        }
  
                        None
                    });
    
                    tokio::select! {
                        result = recv => {
                            use std::result::Result::Ok;
                            match result {
                                Ok(data) => {
                                    if let Some(data) = data {
                                        tx.send(data)?;
                                    }
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
