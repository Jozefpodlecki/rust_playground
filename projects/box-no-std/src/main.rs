#![cfg_attr(not(test), no_std)]
#![no_main]
#![windows_subsystem = "console"]
#![feature(ptr_internals)]
#![feature(sized_type_properties)]
#![feature(coerce_unsized)]
#![feature(unsize)]
#![feature(layout_for_ptr)]
#![allow(static_mut_refs)]
#![allow(unused, internal_features)]

#[cfg(test)]
#[macro_use]
extern crate std;

use core::panic::PanicInfo;

use crate::alloc::init_allocator;

extern crate builtins;

mod alloc;
mod boxed;
mod tests;

#[cfg(not(test))]
#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {
    init_allocator();

    tests::run_tests();

    0
}
