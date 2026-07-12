#![no_std]
#![no_main]
#![windows_subsystem = "console"]
#![allow(unused)]

use core::panic::PanicInfo;

extern crate builtins;

use toolkit::{NtDll, ProcessMemoryWriter, ProcessMemoryProtector, println};
use winapi::shared::ntdef::PVOID;

use crate::scenarios::*;

#[global_allocator]
static ALLOCATOR: emballoc::Allocator<8192> = emballoc::Allocator::new();

mod scenarios;
mod utils;

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info:?}");
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {

    case_not_alertable_infinite();

    0
}