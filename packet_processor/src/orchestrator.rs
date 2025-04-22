use std::sync::{Arc, Mutex};

use anyhow::Result;
use crate::app_state::AppState;
use crate::emitter::Emitter;
use crate::handler::Handler;
use crate::processor::Processor;
use crate::producer::Producer;
use inquire::Select;
pub struct Orchestrator {

}

impl Orchestrator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&self) -> Result<()> {
        let handler = Handler::new();
        let producer = Producer::new();
        let emitter = Emitter::new();
        let emitter = Arc::new(emitter);
        let mut processor = Processor::new(producer, handler, emitter);
        let app_state = AppState::new();
    
        processor.start(app_state);
    
        let processor = Arc::new(Mutex::new(processor));

        let options = vec![
            "Start Processor",
            "Stop Processor",
            "Show Status",
            "Exit",
        ];
        
        loop {
            let answer = Select::new("Choose an action:", options.clone()).prompt()?;

            match answer {
                "Start Processor" => {
                    let processor = processor.lock().unwrap();

                    if processor.is_running() {
                        println!("The processor is already running");
                    }

                    println!("Starting processor...");
                }
                "Stop Processor" => {
                    let mut processor = processor.lock().unwrap();

                    println!("Stopping processor...");
                    if !processor.is_running() {
                        processor.stop();
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