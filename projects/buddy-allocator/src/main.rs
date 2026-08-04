#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![windows_subsystem = "console"]
#![allow(unused)]

use core::panic::PanicInfo;

use toolkit::println;

#[cfg(not(test))]
extern crate builtins;

#[cfg(not(test))]
buddy_allocator::buddy_allocator!(1024 * 1024);

#[macro_use]
extern crate alloc;

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    loop {}
}

#[unsafe(no_mangle)]
#[cfg(not(test))]
pub extern "C" fn mainCRTStartup() -> i32 {

    buddy_allocator::stress::run_stress_test(100, BuddyAllocator);

    0
}