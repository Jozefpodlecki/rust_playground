use std::{sync::{atomic::{AtomicBool, Ordering}, Arc}, thread::{sleep, JoinHandle}, time::Duration};

use abi_stable::{external_types::crossbeam_channel::{self, RReceiver, RSender}, std_types::{RBoxError, RResult::{self, ROk}}};
use shared::Service;

pub struct BackgroundService {
    pub(crate) handle: Option<JoinHandle<()>>,
    pub(crate) tx: Option<RSender<i64>>,
    pub(crate) close_flag: Arc<AtomicBool>, 
}

impl Service for BackgroundService {
    
    fn start(&mut self) -> RResult<RReceiver<i64>, RBoxError> {

        let mut it = 1;
        // let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<i64>();
        let (tx, rx) = crossbeam_channel::unbounded();

        // self.tx = Some(tx.clone());
        let close_flag = self.close_flag.clone();
        let handle = std::thread::spawn(move || {
            
            loop {
                if close_flag.load(Ordering::Relaxed) {
                    break;
                }

                match tx.send(it) {
                    Err(err) => {
                        println!("{}", err)
                    },
                    _ => {}
                }
    
                it += 1;
    
                sleep(Duration::from_secs(1));
            }
        });

        self.handle = Some(handle);

        ROk(rx)
    }

    fn stop(&mut self) -> RResult<(), RBoxError> {

        if let Some(handle) = self.handle.take() {
            self.close_flag.store(true, Ordering::Relaxed);
            handle.join().unwrap();
        }

        ROk(())
    }
}