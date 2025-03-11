use anyhow::Result;
use processor::Processor;
use simple_logger::SimpleLogger;

mod processor;

#[tokio::main]
async fn main() -> Result<()> {
    
    SimpleLogger::new().env().init().unwrap();

    let processor = Processor::new();

    Ok(())
}