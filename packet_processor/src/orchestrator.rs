use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use log::info;
use rand::{rng, Rng};
use tokio::sync::Mutex;
use crate::app_state::AppState;
use crate::emitter::Emitter;
use crate::handler::Handler;
use crate::models::Settings;
use crate::process_checker::{ProcessChecker, ProcessStatus};
use crate::processor::Processor;
use crate::producer::Producer;
use crate::settings_manager::SettingsManager;
use crate::source::Source;
use inquire::Select;
pub struct Orchestrator {

}

impl Orchestrator {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run(&self) -> Result<()> {
        let process_checker = ProcessChecker::new();
        let process_checker = Arc::new(Mutex::new(process_checker));
        let settings_manager = SettingsManager::new();
        let settings_manager = Arc::new(Mutex::new(settings_manager));
        let handler = Handler::new();
        let handler = Arc::new(handler);
        let source = Source::new();
        let producer = Producer::new(source);
        let producer = Arc::new(Mutex::new(producer));
        let emitter = Emitter::new();
        let emitter = Arc::new(emitter);
        let mut processor = Processor::new(
            process_checker.clone(),
            settings_manager.clone(),
            producer,
            handler,
            emitter);
    
        let processor = Arc::new(Mutex::new(processor));

        let options = vec![
            "Start Processor",
            "Stop Processor",
            "Process Checker - Stop",
            "Process Checker - Running",
            "Change Settings (port)",
            "Change Settings",
            "Show Status",
            "Exit",
        ];
        
        loop {
            let answer = Select::new("Choose an action:", options.clone()).prompt()?;

            match answer {
                "Start Processor" => {
                    let mut processor = processor.lock().await;

                    if processor.is_running() {
                        println!("The processor is already running");
                    }
                    else {
                        println!("Starting processor...");
                        let app_state = AppState::new();
                        processor.start(app_state).await;
                    }
                }
                "Process Checker - Stop" => {
                    let mut process_checker = process_checker.lock().await;
                    process_checker.update(ProcessStatus::Stopped)?;
                }
                "Process Checker - Running" => {
                    let mut process_checker = process_checker.lock().await;
                    process_checker.update(ProcessStatus::Stopped)?;
                }
                "Change Settings (port)" => {
                    let mut settings_manager = settings_manager.lock().await;
                    let mut settings = Settings::default();
                    settings.port = rng().random_range(4000..8000);
                    settings_manager.save(settings);
                }
                "Change Settings" => {
                    let mut settings_manager = settings_manager.lock().await;
                    let mut settings = Settings::default();
                    settings.summary_emit_interval = Duration::from_secs(rng().random_range(5..10));
                    settings_manager.save(settings);
                }
                "Stop Processor" => {
                    let mut processor = processor.lock().await;

                    if processor.is_running() {
                        println!("Stopping processor...");
                        processor.stop().await;
                    }
                    
                }
                "Show Status" => {
                    println!("Showing the status...");
                }
                "Exit" => {
                    println!("Exiting...");
                    break;
                }
                _ => println!("Unexpected option selected"),
            }
        }

        Ok(())
    }
}