use anyhow::*;
use orchestrator::Orchestrator;
use simple_logger::SimpleLogger;

mod app_state;
mod processor;
mod producer;
mod handler;
mod models;
mod emitter;
mod orchestrator;

#[tokio::main]
async fn main() -> Result<()> {
    
    SimpleLogger::new().env().init().unwrap();

    let orchestrator = Orchestrator::new();

    orchestrator.run()?;

    Ok(())
}