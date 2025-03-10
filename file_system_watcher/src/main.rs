use simple_logger::SimpleLogger;
use utils::{create_context, run};
use log::*;

mod utils;

fn main() {
    SimpleLogger::new().env().init().unwrap();
    
    match create_context().and_then(run) {
        Err(error) => error!("Error: {error:?}"),
        _ => {}
    }
}
