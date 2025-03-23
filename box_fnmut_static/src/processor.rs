use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

pub type Callback = Arc<Mutex<dyn FnMut(f64) + Send + 'static>>;

pub struct Processor {
    callback: Callback,
}

fn increment(counter: Arc<Mutex<f64>>, callback_placeholder: Arc<Mutex<Option<Callback>>>) -> impl FnMut(f64) {
    move |x: f64| {
        let counter = counter.clone();
        let callback_clone = {
            let lock = callback_placeholder.lock().unwrap();
            lock.as_ref().unwrap().clone()
        };

        thread::spawn(move || {
            {
                let mut counter = counter.lock().unwrap();
                *counter += x;
                println!("Counter: {}", *counter);
            }

            thread::sleep(Duration::from_millis(500));

            let mut callback = callback_clone.lock().unwrap();
            callback(x);
        });
    }
}

impl Processor {
    pub fn new() -> Self {
        let counter = Arc::new(Mutex::new(0.0));
        let callback_placeholder: Arc<Mutex<Option<Callback>>> = Arc::new(Mutex::new(None));

        let callback = {
            let counter_clone = Arc::clone(&counter);
            let callback_placeholder_clone = Arc::clone(&callback_placeholder);

            increment(counter_clone, callback_placeholder_clone)
        };

        let callback_arc: Callback = Arc::new(Mutex::new(callback));
        *callback_placeholder.lock().unwrap() = Some(callback_arc.clone());

        Processor { callback: callback_arc }
    }

    pub fn start(&self, x: f64) {
        let mut cb = self.callback.lock().unwrap();
        cb(x);
    }
}