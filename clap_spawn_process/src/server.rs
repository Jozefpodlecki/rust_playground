use std::{path::Path, time::Duration};

use interprocess::os::windows::named_pipe::{pipe_mode, tokio::{DuplexPipeStream, PipeListenerOptionsExt}, PipeListenerOptions};
use log::*;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpListener, time::{self, sleep}};
use anyhow::Result;

pub struct Server {

}

impl Server {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run(&self, port: u16) -> Result<()> {
        
        let address = format!("127.0.0.1:{}", port);
    
        let listener = TcpListener::bind(&address).await?;
        info!("Server running on {}", address);

        loop {
            match listener.accept().await {
                Ok((mut stream, addr)) => {
                    info!("New client connected: {}", addr);
                    
                    tokio::spawn(async move {
                        Self::setup_ipc_server_and_relay_to_client(&mut stream).await;
                    });
                }
                Err(err) => error!("Failed to accept connection: {}", err),
            }
        }
    }

    pub async fn setup_ipc_server_and_relay_to_client(stream: &mut tokio::net::TcpStream) {
        static PIPE_NAME: &str = "Collector";

        let pipe_path = format!("\\\\.\\pipe\\{}", PIPE_NAME);
        let pipe_path = Path::new(pipe_path.as_str());
    
        let listener = PipeListenerOptions::new()
            .path(pipe_path)
            .create_tokio_duplex::<pipe_mode::Bytes>().unwrap();

        info!("Accepting data at {}", pipe_path.display());

        loop {
            let connection = match listener.accept().await {
                Ok(c) => c,
                Err(e) => {
                    error!("There was an error with an incoming connection: {e}");
                    continue;
                }
            };
    
            if let Err(e) = Self::relay_to_client(connection, stream).await {
                error!("error while handling connection: {e}");
            }
        }

    }

    async fn relay_to_client(connection: DuplexPipeStream<pipe_mode::Bytes>, stream: &mut tokio::net::TcpStream) -> anyhow::Result<()> {
        let (mut recver, mut sender) = connection.split();
    
        let mut buffer = [0u8; 128];
        let duration = Duration::from_secs(1);
    
        loop {
            let bytes_read = recver.read(&mut buffer).await?;
    
            if bytes_read == 0 {
                error!("apparently disconnected");
                drop((recver, sender));
                return Ok(())
            }
    
            stream.write_all(&buffer[..bytes_read]).await?;
            sleep(duration).await;
        }
    
        Ok(())
    }
}