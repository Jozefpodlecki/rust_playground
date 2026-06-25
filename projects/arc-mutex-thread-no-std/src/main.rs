#![no_std]
#![no_main]
#![windows_subsystem = "console"]
#![feature(generic_atomic)]
#![feature(allocator_api)]
#![allow(invalid_reference_casting)]

use core::panic::PanicInfo;

use utils::*;

use crate::{arc::Arc, mutex::Mutex, rwlock::RwLock, thread::Thread};

mod mutex;
mod rwlock;
mod arc;
mod thread;

#[global_allocator]
static ALLOCATOR: emballoc::Allocator<8192> = emballoc::Allocator::new();

#[macro_use]
extern crate alloc;

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {
    let shared_data = Arc::new(RwLock::new(42));

    let shared_data_clone = shared_data.clone();
    let handle = Thread::spawn(move || {
        loop {
            
            {
                let mut shared_data = shared_data_clone.write();
                *shared_data += 1;
            }

            Sleeper::sleep(500);
        }
    });

    let handle = match handle {
        Ok(handle) => handle,
        Err(status) => return status,
    };

    loop {
        let shared_data = shared_data.read();
        println!("{}", *shared_data);
        Sleeper::sleep(500);
    }

    if let Err(err) = handle.join() {
        println!("error")
    }
   
    0
}