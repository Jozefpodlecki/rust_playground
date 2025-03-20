use std::{os::windows::thread, sync::{atomic::{AtomicBool, Ordering}, Arc}, thread::{sleep, JoinHandle}, time::Duration};

use abi_stable::{export_root_module, external_types::crossbeam_channel::{self, RReceiver, RSender}, sabi_extern_fn, sabi_trait::TD_Opaque, std_types::{RBoxError, RResult::{self, ROk}}};
use shared::{Service, ServiceRoot, ServiceRoot_Prefix, ServiceRoot_Ref, ServiceType, Service_TO};
use abi_stable::prefix_type::PrefixTypeTrait;
use tokio::{sync::mpsc::UnboundedReceiver, task::{self}};

#[export_root_module]
fn instantiate_root_module() -> ServiceRoot_Ref {
    ServiceRoot { new }.leak_into_prefix()
}

#[sabi_extern_fn]
pub fn new() -> RResult<ServiceType, RBoxError> {
    let this = BackgroundService { 
        tx: None,
        handle: None,
        close_flag: Arc::new(AtomicBool::new(false))
     };
    ROk(Service_TO::from_value(this, TD_Opaque))
}

struct BackgroundService {
    handle: Option<JoinHandle<()>>,
    tx: Option<RSender<i64>>,
    close_flag: Arc<AtomicBool>, 
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