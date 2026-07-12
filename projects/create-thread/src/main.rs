#![no_std]
#![no_main]
#![windows_subsystem = "console"]
#![allow(static_mut_refs)]
#![feature(arbitrary_self_types_pointers)]
#![allow(unused)]

use core::panic::PanicInfo ;

use ntapi::ntpsapi::NtCurrentThreadId;
use toolkit::{Sleeper, println};

extern crate builtins;

mod arena;
mod packet;

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

thread_allocator!(2048);

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {
   
    let handle  = create_thread(move || {
        let mut count = 0;
        
        loop {
            count += 1;
            Sleeper::sleep(1000);
        }

        42
    }).unwrap();

    match handle.join() {
        Ok(result) => println!("{result}"),
        Err(e) => println!("Thread error: {:?}", e),
    }

    Sleeper::sleep(1000000);

    0
}