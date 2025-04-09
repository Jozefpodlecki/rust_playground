use std::time::Duration;

use log::*;
use tokio::{io::AsyncWriteExt, net::TcpListener, time};
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
                        Self::send_to_client(&mut stream, addr).await;
                    });
                }
                Err(err) => error!("Failed to accept connection: {}", err),
            }
        }
    }

    pub async fn send_to_client(stream: &mut tokio::net::TcpStream, addr: std::net::SocketAddr) {
        let mut interval = time::interval(Duration::from_secs(2));

        loop {
            interval.tick().await;
            
            let encoded = vec![1,2,3];
    
            if let Err(e) = stream.write_all(&encoded).await {
                error!("Failed to send message to {}: {}", addr, e);
                break;
            }
    
        }
    }
}