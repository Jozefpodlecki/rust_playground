use std::{path::Path, time::Duration};

use interprocess::os::windows::named_pipe::{pipe_mode, tokio::{DuplexPipeStream, PipeListenerOptionsExt}, PipeListenerOptions};
use log::*;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpListener, time::sleep};
use anyhow::Result;

use crate::models::Payload;

pub struct Server {

}

impl Server {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run(&self, ip_address: String, port: u16, pipe_name: String) -> Result<()> {
        
        let address = format!("{}:{}", ip_address, port);
    
        let listener = TcpListener::bind(&address).await?;
        info!("Server running on {}", address);

        loop {
            match listener.accept().await {
                Ok((mut stream, addr)) => {
                    info!("New client connected: {}", addr);
                    // let pipe_name = pipe_name.clone();
                    let message = b"hello\n";
                    tokio::spawn(async move {
                        // Self::setup_ipc_server_and_relay_to_client(&pipe_name, &mut stream).await;
                        loop {
                            stream.write_all(message).await.unwrap();
                            sleep(Duration::from_secs(1)).await
                        }
                    });
                }
                Err(err) => error!("Failed to accept connection: {}", err),
            }
        }
    }

    // pub async fn setup_ipc_server_and_relay_to_client(pipe_name: &str, stream: &mut tokio::net::TcpStream) {

    //     let pipe_path = format!("\\\\.\\pipe\\{}", pipe_name);
    //     let pipe_path = Path::new(pipe_path.as_str());
    
    //     let listener = PipeListenerOptions::new()
    //         .path(pipe_path)
    //         .create_tokio_duplex::<pipe_mode::Bytes>().unwrap();

    //     info!("Accepting data at {}", pipe_path.display());

    //     loop {
    //         let connection = match listener.accept().await {
    //             Ok(connection) => connection,
    //             Err(err) => {
    //                 error!("There was an error with an incoming connection: {err}");
    //                 continue;
    //             }
    //         };
    
    //         if let Err(err) = Self::relay_to_client(connection, stream).await {
    //             error!("error while handling connection: {err}");
    //         }
    //     }

    // }

    // async fn relay_to_client(connection: DuplexPipeStream<pipe_mode::Bytes>, stream: &mut tokio::net::TcpStream) -> anyhow::Result<()> {
    //     let (mut recver, sender) = connection.split();
    
    //     let mut buffer = [0u8; 128];
    
    //     loop {
    //         let bytes_read = recver.read(&mut buffer).await?;
    
    //         if bytes_read == 0 {
    //             debug!("Client disconnected, shutting down.");
    //             drop((recver, sender));
    //             return Ok(())
    //         }

    //         let (data, _): (_, usize) = bincode::decode_from_slice::<Payload, _>(&buffer[..bytes_read], bincode::config::standard())?;
    //         println!("{:?}", data);
    
    //         stream.write_all(&buffer[..bytes_read]).await?;
    //     }
    // }
}