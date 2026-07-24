#![no_std]
#![no_main]
#![windows_subsystem = "console"]
#![feature(generic_atomic)]
#![feature(allocator_api)]
#![allow(invalid_reference_casting)]
#![allow(unused)]

use core::panic::PanicInfo;

extern crate builtins;

use toolkit::println;
use winapi::{um::{minwinbase::EXCEPTION_INT_DIVIDE_BY_ZERO, winnt::EXCEPTION_POINTERS}, vc::excpt::EXCEPTION_CONTINUE_SEARCH};

use crate::{arc_mutex::ArcMutex, rwlock::RwLock, thread::Thread};

mod rwlock;
mod arc;
mod arc_mutex;
mod thread;

#[global_allocator]
static ALLOCATOR: emballoc::Allocator<8192> = emballoc::Allocator::new();

#[macro_use]
extern crate alloc;

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info:?}");
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {

    let handle = toolkit::Thread::spawn_ex(move || {
         fn recurse(depth: usize) {
            let rsp: u64;
            unsafe {
                core::arch::asm!("mov {}, rsp", out(reg) rsp);
            }
            println!("Depth: {}, RSP: 0x{:X}", depth, rsp);

            let mut buf = [0u8; 4096]; // 4KB per frame
            buf[0] = depth as u8;
            // recurse(depth + 1);
        }
        recurse(0);

        // loop {}
    }).unwrap();

    // let handle = match handle {
    //     Ok(handle) => handle,
    //     Err(status) => return status,
    // };


    // loop {
    //     let shared_data = shared_data.read();
    //     println!("{}", *shared_data);
    //     Sleeper::sleep(500);
    // }

    if let Err(err) = handle.join() {
        println!("error {err}")
    }
   
    0
}