#![allow(warnings)]

use core::error;
use std::{error::Error, io::{self, Read, Write}, net::Ipv4Addr, sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex}, time::Duration, vec};

use anyhow::{Result};
use log::{error, info};
use simple_logger::SimpleLogger;
use wrapper::NpCapWrapper;

mod wrapper;

pub fn pause() -> Result<()> {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    write!(stdout, "Press any key to continue...")?;
    stdout.flush().unwrap();

    let _ = stdin.read(&mut [0u8])?;

    Ok(())
}

fn run() -> Result<()> {
    let mut npcap_wrapper = NpCapWrapper::new()?;

    let port = 443;
    let rx = npcap_wrapper.start(port)?;
    let mut it = 0;

    let duration = Duration::from_secs(1);
    loop {
        match rx.recv_timeout(duration) {
            Ok(_data) => {
                if it > 5 {
                    npcap_wrapper.stop();
                }

                it += 1;
            },
            Err(_) => continue,
        }
    }

    npcap_wrapper.wait();
}

#[tokio::main]
async fn main() -> Result<()> {
    
    SimpleLogger::new().env().init().unwrap();

    match run() {
        Ok(_) => {
            
        },
        Err(err) => error!("{}", err),
    };

    pause()?;

    Ok(())
}