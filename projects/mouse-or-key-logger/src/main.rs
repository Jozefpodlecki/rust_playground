#![no_std]
#![no_main]
#![windows_subsystem = "console"]
#![allow(unused)]

use core::{panic::PanicInfo, ptr};

#[global_allocator]
static ALLOCATOR: emballoc::Allocator<8192> = emballoc::Allocator::new();

#[macro_use]
extern crate alloc;

mod set_windows_hook;
mod raw_input_device;

extern crate builtins;

use crate::set_windows_hook::run_scenario_set_windows_hook;
use crate::raw_input_device::run_scenario_raw_input_device;

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {
    
    // run_scenario_set_windows_hook();
    run_scenario_raw_input_device();

    0
}