#![no_std]
#![no_main]
#![windows_subsystem = "console"]
#![allow(static_mut_refs)]
#![allow(unused)]

use core::panic::PanicInfo;

use utils::{NtConsole, U16CStackString, println};

use crate::process::{SystemProcessIterator, enable_debug_privilege};

mod process;

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {

    // enable_debug_privilege();
    
    let mut iter = SystemProcessIterator::new().unwrap();
    
    let count = iter.total_count();

    for process in iter.take(20) {
        println!(
            "PID={}, Threads={}, Handles={}, Name='{}', PathAdm='{}', PathInf='{}'",
            process.pid(),
            process.thread_count(),
            process.handle_count(),
            process.name(),
            process.path_req_admin(),
            process.path_via_spiinf()
        );
    }

    0
}