
use std::{fs::File, io::{BufReader, Read, Write}, path::{Path, PathBuf}, time::Duration};

use anyhow::{Ok, Result};
use consumer::Consumer;
use log::*;
use simple_logger::SimpleLogger;
use utils::pause;

mod utils;
mod consumer;

async fn run() -> Result<()> {

    let path = Path::new("dump.data");

    if path.exists() {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        loop {
            let mut len_buf = [0u8; 4];
            if reader.read_exact(&mut len_buf).is_err() {
                break;
            }
    
            let len = u32::from_le_bytes(len_buf) as usize;
            let mut chunk = vec![0u8; len];
            reader.read_exact(&mut chunk)?;
        }

        return Ok(());
    }
  
    let ip_address = "127.0.0.1"; 
    let port = 443;
    let mut consumer = Consumer::new();
    let mut rx = consumer.start(ip_address, port).await?; 
    let mut file = File::create(path)?;
        
    loop {
        let data = match rx.recv().await {
            Some(data) => data,
            None => break,
        };

        let len = data.len() as u32;
        file.write_all(&len.to_le_bytes())?; 
        file.write_all(&data)?;   
    }

    file.flush()?;

    consumer.stop()?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    
    SimpleLogger::new().env().init().unwrap();

    match run().await {
        std::result::Result::Ok(_) => {
            info!("main:run:Ok");
        },
        Err(err) => {
            error!("{}", err);
        },
    };

    pause();

    Ok(())
}