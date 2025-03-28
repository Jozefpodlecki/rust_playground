use std::{net::SocketAddr, time::Duration};

use log::*;
use tokio::{io::AsyncWriteExt, net::TcpStream, time};


pub async fn send_to_client(stream: &mut TcpStream, addr: SocketAddr) {
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
