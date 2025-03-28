use log::*;
use simple_logger::SimpleLogger;
use utils::send_to_client;
use tokio::net::TcpListener;
use anyhow::Result;
use clap::{arg, command, Parser};
mod utils;

/// Command-line arguments for the server
#[derive(Parser)]
#[command(name = "Rust TCP Server")]
#[command(about = "A simple async TCP server with logging")]
struct Args {
    /// Port to listen on (default: 6041)
    #[arg(short, long, default_value_t = 6041)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    SimpleLogger::new().env().init().unwrap();
    let args = Args::parse();
    let address = format!("127.0.0.1:{}", args.port);
    
    let listener = TcpListener::bind(&address).await?;
    info!("Server running on {}", address);

    loop {
        match listener.accept().await {
            Ok((mut stream, addr)) => {
                info!("New client connected: {}", addr);
                
                tokio::spawn(async move {
                    send_to_client(&mut stream, addr).await;
                });
            }
            Err(e) => error!("Failed to accept connection: {}", e),
        }
    }
}