use log::{error, info};
use models::{Message, UpdateStatus};
use process_watcher::ProcessWatcher;
use processor::Processor;
use simple_logger::SimpleLogger;
use update_checker::UpdateChecker;
use anyhow::*;

mod aws_iprange;
mod process_watcher;
mod models;
mod processor;
mod update_checker;
mod emitter_listener;
mod hook;

async fn runner() -> Result<()> {
    let (tx, rx) = std::sync::mpsc::channel::<Message>();
    let mut update_checker = UpdateChecker::new();
    let mut processor = Processor::new();
    let mut process_watcher = ProcessWatcher::new();
    
    let update_result = update_checker.start();
    
    if let UpdateStatus::NewVersion = update_result {
        return Ok(());
    }
    
    let process_name = "client_server.exe";
    let port = 6041;
    process_watcher.start(process_name, port, tx.clone());

    loop {
        let message = rx.recv().unwrap();
        
        info!("{:?}", message);

        match message {
            Message::ProcessListening(_) => {
                processor.start();
            },
            Message::ProcesStopped => {
                processor.stop()?;
            },
            _ => {}
        }

    }

    Ok(())
}

#[tokio::main]
async fn main() {
    hook::set_hook();

    SimpleLogger::new().env().init().unwrap();
   
   match runner().await {
    std::result::Result::Ok(_) => {
        info!("finished");
    },
    Err(err) => {
        error!("{}", err);
    },
   }
}
