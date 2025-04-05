use std::{sync::{atomic::{AtomicBool, Ordering}, Arc}, thread::{sleep, JoinHandle}, time::Duration};

use abi_stable::std_types::{RBoxError, ROption::RNone, RResult::{self, ROk}};
use rand::{rng, Rng, RngCore};
use shared::{models::Command, Service, TokioMpscWrapper, TokioService};
use tokio::sync::mpsc::{self, UnboundedReceiver};

pub struct TokioBackgroundService {
    pub(crate) handle: Option<JoinHandle<()>>,
    pub(crate) tx: Option<UnboundedReceiver<i64>>,
    pub(crate) close_flag: Arc<AtomicBool>, 
}

impl TokioService for TokioBackgroundService {

    fn start_v2(&mut self) -> TokioMpscWrapper {
        println!("start_v2");

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Command>();

        let close_flag = self.close_flag.clone();
        let handle = std::thread::spawn(move || {
            
            let mut rng = rng();

            loop {
                if close_flag.load(Ordering::Relaxed) {
                    break;
                }

                let command = match rng.random_range(1..=3) {
                    1 => Command::Insert { id: rng.next_u64(), name: "name".into(), value: "value".into() },
                    2 => Command::Update { id: rng.next_u64(), name: RNone },
                    3 => Command::Delete { id: rng.next_u64() },
                    _ => panic!("Invalid")
                };

                match tx.send(command) {
                    Err(err) => {
                        println!("{}", err)
                    },
                    _ => {}
                }
    
                sleep(Duration::from_secs(1));
            }
        });

        self.handle = Some(handle);

        TokioMpscWrapper::new(rx)
    }
    
    fn start(&mut self) -> *mut () {

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Command>();

        let close_flag = self.close_flag.clone();
        let handle = std::thread::spawn(move || {
            
            let mut rng = rng();

            loop {
                if close_flag.load(Ordering::Relaxed) {
                    break;
                }

                let command = match rng.random_range(1..=3) {
                    1 => Command::Insert { id: rng.next_u64(), name: "name".into(), value: "value".into() },
                    2 => Command::Update { id: rng.next_u64(), name: RNone },
                    3 => Command::Delete { id: rng.next_u64() },
                    _ => panic!("Invalid")
                };

                match tx.send(command) {
                    Err(err) => {
                        println!("{}", err)
                    },
                    _ => {}
                }
    
                sleep(Duration::from_secs(1));
            }
        });

        self.handle = Some(handle);

        
        Box::into_raw(Box::new(rx)) as *mut ()
    }

    fn stop(&mut self) -> RResult<(), RBoxError> {

        if let Some(handle) = self.handle.take() {
            self.close_flag.store(true, Ordering::Relaxed);
            handle.join().unwrap();
        }

        ROk(())
    }
}