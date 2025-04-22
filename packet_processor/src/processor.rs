use std::{sync::Arc, thread};

use log::*;
use tokio::{runtime::Runtime, sync::watch};

use crate::{app_state::AppState, emitter::Emitter, handler::Handler, producer::Producer};


pub struct Processor {
    handle: Option<thread::JoinHandle<Result<AppState, anyhow::Error>>>,
    producer: Producer,
    handler: Option<Handler>,
    emitter: Arc<Emitter>,
    shutdown_tx: Option<watch::Sender<()>>,
}

impl Processor {
    pub fn new(
        producer: Producer,
        handler: Handler,
        emitter: Arc<Emitter>) -> Self {
        Self {
            handle: None,
            producer,
            emitter,
            handler: Some(handler),
            shutdown_tx: None
        }
    }

    pub fn is_running(&self) -> bool {
        self.handle.is_some()
    }

    pub fn start(&mut self, mut state: AppState) {
      
        let mut rx = self.producer.start();
        let handler = self.handler.take().expect("Handler unset");

        let (shutdown_tx, mut shutdown_rx) = watch::channel(());
        self.shutdown_tx = Some(shutdown_tx);

        let emitter = self.emitter.clone();

        let handle = thread::spawn(move || {
            let runtime = Runtime::new()?;
          
            runtime.block_on(async {
                
                loop {
                    tokio::select! {
                        data = rx.recv() => {
                            let data = match data {
                                Some(data) => data,
                                None => break,
                            };

                            handler.handle(&data, &mut state).await?;

                            
                        }
    
                        _ = shutdown_rx.changed() => {
                            break;
                        }
                    }
                }

                anyhow::Ok(state)
            })
        });

        self.handle = Some(handle);

    }

    pub fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    
        if let Some(handle) = self.handle.take() {
            match handle.join() {
                Ok(result) => {
                    if let Err(err) = result {
                        error!("Processor error: {:?}", err);
                    }
                }
                Err(err) => {
                    error!("Failed to join thread: {:?}", err);
                }
            }
        }
    }
}
