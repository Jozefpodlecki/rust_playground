use std::{sync::{self, atomic::{AtomicBool, Ordering}, mpsc::{Receiver, Sender}, Arc}, thread::{self, JoinHandle}, time::Duration};

use anyhow::{Error, Ok, Result};
use log::*;
use pcap::{Active, Capture, Device};

pub struct NpCapWrapper {
    handle: Option<JoinHandle<()>>,
    close_flag: Arc<AtomicBool>,
}

impl NpCapWrapper {
    pub fn new() -> Result<Self> {
      
        Ok(Self {
            handle: None,
            close_flag: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn start(&mut self, port: u16) -> Result<Receiver<Vec<u8>>> {

        if self.handle.is_some() {
            return Err(anyhow::anyhow!("Capture is already running").into());
        }

        let interface = default_net::get_default_interface()
            .map_err(|err| anyhow::anyhow!("Could not get default interface: {:?}", err))?;
 
        let device = Device::list()?
            .iter()
            .find(|device| device.name.contains(&interface.name))
            .cloned()
            .unwrap();

        debug!("Using {}", device.desc.as_ref().unwrap_or_else(|| &device.name));
        let mut capture = Capture::from_device(device)?
            .timeout(Duration::from_secs(1).as_millis() as i32)
            .open()?;
        let filter = format!("tcp port {port}");

        capture.filter(&filter, true)?;

        let (tx, rx) = sync::mpsc::channel::<Vec<u8>>();
        let close_flag: Arc<AtomicBool> = self.close_flag.clone();

        let handle = thread::spawn(|| Self::listen(close_flag, capture, tx));
        self.handle = Some(handle);
        
        Ok(rx)
    }

    pub fn listen(close_flag: Arc<AtomicBool>, mut capture: Capture<Active>, tx: Sender<Vec<u8>>) {
        loop {
            if close_flag.load(Ordering::Relaxed) {
                break;
            }

            match capture.next_packet() {
                std::result::Result::Ok(packet) => {
                    tx.send(packet.data.to_owned()).unwrap();
                    println!("received len {:?}", packet.data.len());
                },
                Err(err) => {
                    match err {
                        pcap::Error::TimeoutExpired => continue,
                        err => println!("{:?}", err)
                    }
                },
            }
        }
    }

    pub fn stop(&mut self) {
        self.close_flag.store(true, Ordering::Relaxed);
    }

    pub fn wait(&mut self) {

        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }

    }
}