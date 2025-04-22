use std::{sync::Arc, thread};

use log::*;
use tokio::{runtime::Runtime, sync::{watch, Mutex}};

use crate::{app_state::AppState, emitter::Emitter, handler::Handler, interval_timer::IntervalTimer, producer::Producer, settings_manager::SettingsManager};


pub struct Processor {
    settings_manager: Arc<Mutex<SettingsManager>>,
    producer: Producer,
    handler: Option<Handler>,
    emitter: Arc<Emitter>,
    shutdown_tx: Option<watch::Sender<()>>,
    handle: Option<thread::JoinHandle<Result<AppState, anyhow::Error>>>,
}

impl Processor {
    pub fn new(
        settings_manager: Arc<Mutex<SettingsManager>>,
        producer: Producer,
        handler: Handler,
        emitter: Arc<Emitter>) -> Self {
        Self {
            settings_manager,
            handle: None,
            producer,
            emitter,
            handler: Some(handler),
            shutdown_tx: None
        }
    }

    pub fn is_running(&self) -> bool {
        if let Some(handle) = &self.handle {
            return handle.is_finished();
        }

        false
    }

    pub async fn start(&mut self, mut state: AppState) {
      
        let handler = self.handler.take().expect("Handler unset");

        let (shutdown_tx, mut shutdown_rx) = watch::channel(());
        self.shutdown_tx = Some(shutdown_tx);

        let settings_manager= self.settings_manager.clone();
        let mut settings = settings_manager.lock().await.get().clone();
        let emitter = self.emitter.clone();
        let mut rx = self.producer.start(settings.port);

        let handle = thread::spawn(move || {
            let runtime = Runtime::new()?;
          
            runtime.block_on(async {

                let mut settings_rx = settings_manager.lock().await.subscribe();
                let mut interval_timer = IntervalTimer::new(settings.summary_emit_interval);

                loop {
                    tokio::select! {
                        data = rx.recv() => {
                            let data = match data {
                                Some(data) => data,
                                None => break,
                            };

                            handler.handle(&data, &mut state).await?;

                            let summary = state.get_summary(&settings);

                            if interval_timer.tick_if_elapsed() {
                                emitter.emit(summary)?;
                            }
                        }
                        _ = settings_rx.changed() => {
                            info!("settings changed");
                            let new_settings = settings_rx.borrow().clone();

                            if new_settings.port != settings.port {
                                info!("port changed. stopping");
                                break;
                            }

                            settings = new_settings;
                            interval_timer.set_interval(settings.summary_emit_interval);
                        }
                        _ = shutdown_rx.changed() => {
                            info!("shutdown");
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
