use std::{sync::{Arc, Mutex}, thread};


fn main() {
    let data = Arc::new(Mutex::new(0));

    {
        let data_clone = Arc::clone(&data);
        let builder = thread::Builder::new();
        let handle = builder
            .name("mutex panic".into())
            .spawn(move || {
            let _lock = data_clone.lock().unwrap();
            panic!("Thread panicked while holding the mutex!");
        }).unwrap();

        match handle.join() {
            Ok(_) => println!("done"),
            Err(err) => eprintln!("Thread Error: {:?}", err),
        }
    }

    let result = data.lock();
    match result {
        Ok(_) => println!("Lock acquired successfully."),
        Err(_) => println!("Mutex is poisoned!"),
    }
}
