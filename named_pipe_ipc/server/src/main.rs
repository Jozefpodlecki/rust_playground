use std::path::Path;
use std::time::Duration;

use interprocess::os::windows::named_pipe::{pipe_mode, PipeListenerOptions};
use interprocess::os::windows::named_pipe::tokio::{DuplexPipeStream, PipeListenerOptionsExt};
use tokio::io::{self, AsyncReadExt};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    static PIPE_NAME: &str = "Example";

    let pipe_test = Path::new("\\\\.\\pipe\\Example");

    let listener = PipeListenerOptions::new()
        .path(pipe_test)
        .create_tokio_duplex::<pipe_mode::Bytes>()?;
    
    println!(r"Server running at \\.\pipe\{PIPE_NAME}");

    loop {
        let connection = match listener.accept().await {
            Ok(c) => c,
            Err(e) => {
                println!("There was an error with an incoming connection: {e}");
                continue;
            }
        };

        tokio::spawn(async move {
            if let Err(e) = handle_connection(connection).await {
                println!("error while handling connection: {e}");
            }
        });
    }

    Ok(())
}

async fn handle_connection(connection: DuplexPipeStream<pipe_mode::Bytes>) -> io::Result<()> {
    println!("handle_connection");
    let (mut recver, mut sender) = connection.split();

    let mut buffer = [0u8; 128];
    let duration = Duration::from_secs(1);

    loop {
        let bytes_read = recver.read(&mut buffer).await?;
        let message = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("Received: {}", message);
        sleep(duration).await;
    }

    drop((recver, sender));

    Ok(())
}