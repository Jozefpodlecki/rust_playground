#![no_std]
#![no_main]
#![windows_subsystem = "console"]
#![feature(sync_unsafe_cell)]
#![allow(static_mut_refs, non_snake_case, non_camel_case_types)]
#![feature(arbitrary_self_types_pointers)]
#![feature(ptr_alignment_type)]

use core::panic::PanicInfo;

use alloc::vec::Vec;
use toolkit::println;

mod stress;
mod allocator;
mod buddy;

extern crate builtins;

buddy_allocator!(1024 * 1024);

#[macro_use]
extern crate alloc;

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {

    stress::run_stress_test(100);

    0
}