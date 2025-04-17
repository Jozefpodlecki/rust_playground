#![allow(warnings)]

use std::panic;
use runner::*;

mod models;
mod utils;
mod renderer;
mod multi_thread_simulator;
mod runner;

use anyhow::*;

fn main() -> Result<()> {
    panic::set_hook(Box::new(|panic_info| {
        eprintln!("Panic occurred: {}", panic_info);
    }));

    // run()?;
    run_threaded()?;

    Ok(())
}
