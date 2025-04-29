
use std::{env, fs::File, io::{BufReader, BufWriter, Read, Write}, path::{Path, PathBuf}, time::Duration};

use anyhow::{Ok, Result};
use chrono::Local;
use consumer::Consumer;
use log::*;
use simple_logger::SimpleLogger;
use utils::pause;

mod utils;
mod consumer;

async fn run() -> Result<()> {

    let filter = match env::args().nth(1) {
        Some(filter) => filter,
        None => {
            println!("No arguments");
            return Ok(());
        },
    };

    info!("Filter: {}", filter);

    // let path = Path::new("dump.data");
    // if path.exists() {
    //     let file = File::open(path)?;
    //     let mut reader = BufReader::new(file);

    //     loop {
    //         let mut len_buf = [0u8; 4];
    //         if reader.read_exact(&mut len_buf).is_err() {
    //             break;
    //         }
    
    //         let len = u32::from_le_bytes(len_buf) as usize;
    //         let mut chunk = vec![0u8; len];
    //         reader.read_exact(&mut chunk)?;
    //         info!("{} {:?}", len, chunk);
    //     }

    //     return Ok(());
    // }
  
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("dump_{}.data", timestamp);
    let path = Path::new(&filename);
    let abs_path = env::current_dir()?.join(path);

    let ip_address = "127.0.0.1"; 
    let port = 6040;
    // info!("Listening on port {}", port);
    let mut consumer = Consumer::new();
    let mut rx = consumer.start(filter.to_string()).await?; 
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    info!("Output will be saved to: {}", abs_path.display());

    loop {
        tokio::select! {
            data = rx.recv() => {
                if let Some(data) = data {
                    let len = data.len() as u32;
                    info!("received {} bytes", len);
                    writer.write_all(&len.to_le_bytes())?;
                    writer.write_all(&data)?;
                } else {
                    break;
                }
            }
            _ = tokio::signal::ctrl_c() => {
                info!("Received Ctrl+C, shutting down.");
                writer.flush()?;
                consumer.stop()?;
                break;
            }
        }
    }

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