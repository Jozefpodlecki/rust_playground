use std::{thread, time::Duration};

use shared::Payload;
use tokio::sync::mpsc::{self, UnboundedReceiver};

#[unsafe(no_mangle)]
pub extern "C" fn tokio_mpsc() -> *mut UnboundedReceiver<Payload> {
    let (tx, rx) = mpsc::unbounded_channel::<Payload>();

    std::thread::spawn(move || {

        loop {
            thread::sleep(Duration::from_secs(2));
            tx.send(Payload::random()).unwrap();
        }
    });

    Box::into_raw(Box::new(rx))
}