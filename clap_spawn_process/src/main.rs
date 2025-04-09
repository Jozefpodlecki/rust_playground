mod models;
mod controller;
mod server;
mod client;

use anyhow::Result;
use clap::Parser;
use controller::Controller;
use log::*;
use models::CommandArgs;
use simple_logger::SimpleLogger;


#[tokio::main]
async fn main() -> Result<()> {
    SimpleLogger::new().env().init().unwrap();

    let args = CommandArgs::parse();
    let controller = Controller::new();
    match controller.run(&args).await {
        Ok(_) => {
            info!("Process {:?} completed successfully.", args.r#type);
        }
        Err(e) => {
            error!("Error: {}", e);
        }
    }
   
    Ok(())
}