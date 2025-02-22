use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

type Callback = Arc<Mutex<dyn FnMut(f64) + Send + 'static>>;

struct Processor {
    callback: Callback,
}

impl Processor {
    fn new() -> Self {
        let counter = Arc::new(Mutex::new(0.0));
        let callback_placeholder: Arc<Mutex<Option<Callback>>> = Arc::new(Mutex::new(None));

        let callback = {
            let counter_clone = Arc::clone(&counter);
            let callback_placeholder_clone = Arc::clone(&callback_placeholder);

            move |x: f64| {
                let counter_clone = Arc::clone(&counter_clone);
                let callback_clone = {
                    let lock = callback_placeholder_clone.lock().unwrap();
                    lock.as_ref().unwrap().clone()
                };

                thread::spawn(move || {
                    {
                        let mut counter = counter_clone.lock().unwrap();
                        *counter += x;
                        println!("Counter: {}", *counter);
                    }

                    thread::sleep(Duration::from_millis(500));

                    let mut cb = callback_clone.lock().unwrap();
                    cb(x);
                });
            }
        };

        let callback_arc: Callback = Arc::new(Mutex::new(callback));
        *callback_placeholder.lock().unwrap() = Some(callback_arc.clone());

        Processor { callback: callback_arc }
    }

    fn start(&self, x: f64) {
        let mut cb = self.callback.lock().unwrap();
        cb(x);
    }
}

fn main() {
    let processor = Processor::new();
    processor.start(1.5);

    thread::sleep(Duration::from_secs(5));
}
