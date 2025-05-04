
use std::env;
use log::*;
use processor::Processor;
use simple_logger::SimpleLogger;
use utils::pause;
use anyhow::*;

mod utils;
mod consumer;
mod processor;

async fn run() -> Result<()> {

    let filter = match env::args().nth(1) {
        Some(filter) => filter,
        None => {
            println!("No arguments");
            return Ok(());
        },
    };

    let mut processor = Processor::new();
    processor.run(filter).await?;

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