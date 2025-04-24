use std::{future, sync::Arc, thread::{self, sleep}, time::Duration};

use log::*;
use tokio::{runtime::Runtime, sync::{mpsc::UnboundedReceiver, watch, Mutex}};

use crate::{app_state::AppState, emitter::Emitter, handler::Handler, interval_timer::IntervalTimer, models::Settings, process_checker::{ProcessChecker, ProcessStatus}, producer::Producer, settings_manager::SettingsManager};


pub struct Processor {
    process_checker: Arc<Mutex<ProcessChecker>>,
    settings_manager: Arc<Mutex<SettingsManager>>,
    producer: Arc<Mutex<Producer>>,
    handler: Arc<Handler>,
    emitter: Arc<Emitter>,
    shutdown_tx: Option<watch::Sender<()>>,
    handle: Option<thread::JoinHandle<Result<(), anyhow::Error>>>,
}

impl Processor {
    pub fn new(
        process_checker: Arc<Mutex<ProcessChecker>>,
        settings_manager: Arc<Mutex<SettingsManager>>,
        producer: Arc<Mutex<Producer>>,
        handler: Arc<Handler>,
        emitter: Arc<Emitter>) -> Self {
        Self {
            process_checker, 
            settings_manager,
            handle: None,
            producer,
            emitter,
            handler: handler,
            shutdown_tx: None
        }
    }

    pub fn is_running(&self) -> bool {
        if let Some(handle) = &self.handle {
            return !handle.is_finished();
        }

        false
    }

    pub async fn start(&mut self) {

        if self.handle.is_some() {
            return;
        }
      
        let (shutdown_tx, shutdown_rx) = watch::channel(());
        self.shutdown_tx = Some(shutdown_tx);

        let handler = self.handler.clone();
        let settings_manager = self.settings_manager.clone();
        let settings = settings_manager.lock().await.get().clone();
        let emitter = self.emitter.clone();
        let producer = self.producer.clone();
        let process_checker = self.process_checker.clone();

        let handle = thread::spawn(move || {
            let runtime = Runtime::new()?;
          
            runtime.block_on(async {
                let state = AppState::new();
                
                Self::process(
                    state,
                    process_checker,
                    producer,
                    emitter,
                    handler,
                    settings_manager,
                    settings,
                    shutdown_rx).await
            })
        });

        self.handle = Some(handle);

    }

    async fn process(
        mut state: AppState,
        process_checker: Arc<Mutex<ProcessChecker>>,
        producer: Arc<Mutex<Producer>>,
        emitter: Arc<Emitter>,
        handler: Arc<Handler>,
        settings_manager: Arc<Mutex<SettingsManager>>,
        mut settings: Settings,
        mut shutdown_rx: tokio::sync::watch::Receiver<()>) -> anyhow::Result<()> {
        let mut producer = producer.lock().await;
        let mut settings_rx = settings_manager.lock().await.subscribe();
        let mut process_status_rx = process_checker.lock().await.subscribe();
        let mut interval_timer = IntervalTimer::new(settings.summary_emit_interval);
        let mut pending = future::pending::<()>();

        loop {
            tokio::select! {
                _ = process_status_rx.changed() => {
                    let process_status = process_status_rx.borrow().clone();

                    match process_status {
                        ProcessStatus::Idle | 
                        ProcessStatus::Stopped | 
                        ProcessStatus::NotFound => {
                            if !producer.is_running() {
                                continue;
                            }
                            
                            info!("process not running - stopping producer");
                            producer.stop();
                        },
                        ProcessStatus::Running |
                        ProcessStatus::Listening => {
                            if producer.is_running() {
                                continue;
                            }

                            info!("process running - starting producer");
                            producer.start(settings.port);
                        },
                    }
                }
                _ = settings_rx.changed() => {
                    info!("settings changed");
                    let new_settings = settings_rx.borrow().clone();

                    let port_changed = new_settings.port != settings.port;

                    settings = new_settings;
                    interval_timer.set_interval(settings.summary_emit_interval);

                    if port_changed {
                        producer.stop();
                        producer.start(settings.port);
                    }
                }
                _ = shutdown_rx.changed() => {
                    info!("shutdown");
                    break;
                }
                data = producer.recv() => {
                    match data {
                        Some(data) => {
                            handler.handle(&data, &mut state).await?;

                            let summary = state.get_summary(&settings);

                            if interval_timer.tick_if_elapsed() {
                                emitter.emit(summary)?;
                            }  
                        },
                        None => sleep(Duration::from_millis(500)),
                    };
                }
            }
        }

        anyhow::Ok(())
    }

    pub async fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }

        self.producer.lock().await.stop();
    
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
