
use std::time::Duration;

use anyhow::{Ok, Result};
use consumer::Consumer;
use log::*;
use simple_logger::SimpleLogger;
use tokio::time::sleep;
use utils::pause;

mod wrapper;
mod utils;
mod consumer;

async fn run() -> Result<()> {
  
    let mut consumer = Consumer::new();

    consumer.start().await?;

    let duration = Duration::from_secs(3);
    info!("Stopping after {:?}", duration);
    sleep(duration).await;

    consumer.stop().await?;

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