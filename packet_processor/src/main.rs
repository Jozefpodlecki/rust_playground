use anyhow::Result;
use processor::Processor;
use simple_logger::SimpleLogger;

mod packet;
mod processor;
mod game_state;
mod models;

#[tokio::main]
async fn main() -> Result<()> {
    
    SimpleLogger::new().env().init().unwrap();

    let processor = Processor::new();

    processor.run();

    Ok(())
}