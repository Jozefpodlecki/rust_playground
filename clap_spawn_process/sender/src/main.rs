use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Ok, Result};
use bincode::{Decode, Encode};
use interprocess::os::windows::named_pipe::{pipe_mode, tokio::DuplexPipeStream};
use log::*;
use simple_logger::SimpleLogger;
use tokio::{io::AsyncWriteExt, time::sleep};


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

    let connection = match DuplexPipeStream::<pipe_mode::Bytes>::connect_by_path(r"\\.\pipe\Collector").await {
        std::result::Result::Ok(connection) => {
            info!("Connected to server.");
            connection
        }
        Err(e) => {
            error!("Failed to connect to server: {}", e);
            return Ok(());
        }
    };

    let (mut recver, mut sender) = connection.split();
    let duration = Duration::from_secs(1);
    let config = bincode::config::standard();

    loop {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let payload = Payload::New {
            id: 1,
            name: format!("test-{}", now),
        };
        let data = bincode::encode_to_vec(payload, config)?;

        sender.write_all(&data).await?;
  
        sleep(duration).await;
    }

    sender.shutdown().await?;
   
    Ok(())
}