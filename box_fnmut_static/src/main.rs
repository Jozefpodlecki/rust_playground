use std::{error::Error, sync::Mutex, thread, time::Duration};

use processor::Processor;

mod processor;

fn main() {
    let processor = Processor::new();
    processor.start(1.5);

    thread::sleep(Duration::from_secs(5));
}
