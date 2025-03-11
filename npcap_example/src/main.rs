#![allow(warnings)]

use std::{error::Error, net::Ipv4Addr, sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex}, vec};

use anyhow::{Result};
use simple_logger::SimpleLogger;
use wrapper::NpCapWrapper;

mod wrapper;

#[tokio::main]
async fn main() -> Result<()> {
    
    SimpleLogger::new().env().init().unwrap();

    let mut npcap_wrapper = NpCapWrapper::new()?;

    npcap_wrapper.start();

    Ok(())
}