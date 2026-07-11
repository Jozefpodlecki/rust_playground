#![no_std]
#![no_main]
#![windows_subsystem = "console"]
#![allow(static_mut_refs)]
#![allow(unused)]

use log::{LevelFilter, info};
use toolkit::println;

use crate::{etw_logger::EventViewerLogger, nt_file_logger::NtFileLogger};

mod etw_logger;
mod nt_file_logger;
mod time;

extern crate builtins;

#[inline(never)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{info}");
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {
    // let logger = NtFileLogger::init(LevelFilter::Debug).unwrap();
    // println!("NtFileLogger::init");
    if let Err(err) = EventViewerLogger::init() {
        println!("Failed to initialize logger: {:?}", err);
        return 1;
    }
    
    // println!("test");
    info!("Application started!");
    log::warn!("This is a warning message");
    log::error!("An error occurred: file not found");
    // log::debug!("Debug message with value: {}", 42);
    
    // drop(logger);
    0
}