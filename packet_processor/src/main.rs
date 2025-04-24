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
mod settings_manager;
mod process_checker;
mod interval_timer;
mod source;
mod utils;
mod hp_log;

#[tokio::main]
async fn main() -> Result<()> {
    
    SimpleLogger::new().env().init().unwrap();

    let orchestrator = Orchestrator::new();

    orchestrator.run().await?;

    Ok(())
}