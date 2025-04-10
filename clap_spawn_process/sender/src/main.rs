use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Ok, Result};
use bincode::{Decode, Encode};
use interprocess::os::windows::named_pipe::{pipe_mode, tokio::DuplexPipeStream};
use log::*;
use simple_logger::SimpleLogger;
use tokio::io::AsyncWriteExt;


#[derive(Debug, Encode, Decode, Clone)]
pub enum Payload {
    New {
        id: u32,
        name: String,
    },
    Update {
        id: u32,
        name: String,
    },
    Delete {
        id: u32,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    SimpleLogger::new().env().init().unwrap();

    let pipe_name = "Collector";
    let pipe_path = format!("\\\\.\\pipe\\{}", pipe_name);
    let connection = match DuplexPipeStream::<pipe_mode::Bytes>::connect_by_path(pipe_path).await {
        std::result::Result::Ok(connection) => {
            info!("Connected to server.");
            connection
        }
        Err(e) => {
            error!("Failed to connect to server: {}", e);
            return Ok(());
        }
    };

    let (recver, mut sender) = connection.split();
    let config = bincode::config::standard();
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let payload = Payload::New {
        id: 1,
        name: format!("test-{}", now),
    };
    debug!("Sending payload: {:?}", payload);
    let data = bincode::encode_to_vec(payload, config)?;

    sender.write_all(&data).await?;

    sender.shutdown().await?;
   
    Ok(())
}