use std::{sync::{atomic::{AtomicBool, Ordering}, mpsc::{self, Receiver}}, thread, time::Duration};
use shared::Payload;


#[unsafe(no_mangle)]
pub extern "C" fn test_mpsc_with_enum() -> *mut Receiver<Payload> {
    let (tx, rx) = mpsc::channel::<Payload>();

    std::thread::spawn(move || {

        loop {
            thread::sleep(Duration::from_secs(2));
            tx.send(Payload::random()).unwrap();
        }
    });

    Box::into_raw(Box::new(rx))
}