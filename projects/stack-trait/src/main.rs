#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#![feature(ptr_metadata, unsize)]
#![windows_subsystem = "console"]
#![allow(unused)]

use core::panic::PanicInfo;

use crate::testing::test_stacked;

mod data_buf;
mod stack_trait;
mod testing;

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {
    test_stacked();

    0
}