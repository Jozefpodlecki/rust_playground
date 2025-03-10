#![allow(warnings)]

use std::{error::Error, net::Ipv4Addr, sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex}, vec};

use anyhow::{Result};
use simple_logger::SimpleLogger;
use windivert::{layer::NetworkLayer, packet::WinDivertPacket, prelude::WinDivertFlags, WinDivert};
use tokio::{select, signal::windows::{self, ctrl_c}, task};
use wrapper::WindivertWrapper;

mod wrapper;

#[tokio::main]
async fn main() -> Result<()> {
    
    SimpleLogger::new().env().init().unwrap();

    let mut windivert = WindivertWrapper::new()?;

    windivert.start();

    Ok(())
}