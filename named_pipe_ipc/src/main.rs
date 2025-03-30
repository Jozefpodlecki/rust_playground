use std::time::{Duration, SystemTime, UNIX_EPOCH};

use interprocess::os::windows::named_pipe::pipe_mode;
use interprocess::os::windows::named_pipe::tokio::DuplexPipeStream;
use tokio::io::AsyncWriteExt;
use tokio::time::{sleep, sleep_until};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connection = DuplexPipeStream::<pipe_mode::Bytes>::connect_by_path(r"\\.\pipe\Example").await?;

    let (mut recver, mut sender) = connection.split();
    let mut buffer = String::with_capacity(128);
    let duration = Duration::from_secs(1);

    loop {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let message = format!("Message at {}", now);
        sender.write_all(message.as_bytes()).await?;
        println!("sending {}", message);
        sleep(duration).await;
    }

    sender.shutdown().await?;
    
    Ok(())
}