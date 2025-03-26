use std::{error::Error, sync::{Arc, Mutex}, thread};

use anyhow::Context;

fn mutex_lock_thread_safety(locker: Mutex<i32>) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    
    let locker = locker.lock()
        .map_err(|err| anyhow::anyhow!("{:?}", err))?;

    Ok(())
}


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
