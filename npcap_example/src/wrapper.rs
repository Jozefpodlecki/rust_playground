use std::thread::{self, JoinHandle};

use anyhow::{Error, Ok, Result};
use pcap::{Capture, Device};

pub struct NpCapWrapper {
    handle: Option<JoinHandle<()>>
}

impl NpCapWrapper {
    pub fn new() -> Result<Self> {
      
        Ok(Self {
            handle: None
        })
    }

    pub fn start(&mut self) -> Result<()> {

        if self.handle.is_some() {
            return Ok(())
        }

        let device = Device::lookup()?.unwrap();
        let mut capture = Capture::from_device(device)?
            .open()?;
        let port = 8080;
        let filter = format!("tcp port {port}");

        capture.filter(&filter, true).unwrap();

        let handle = thread::spawn(move || {

            while let std::result::Result::Ok(packet) = capture.next_packet() {
                println!("received packet! {:?}", packet);
            }
        });

        self.handle = Some(handle);
        
        Ok(())
    }
}