#![no_std]
#![no_main]
#![windows_subsystem = "console"]

use core::panic::PanicInfo;


use toolkit::println;

use crate::{nt_query::search_via_nt_api, types::{AncestorPath, AncestorPaths}, volume::*};

mod utils;
mod types;
mod nt_query;
mod volume;

extern crate builtins;

#[macro_use]
extern crate alloc;

#[global_allocator]
static ALLOCATOR: emballoc::Allocator<{4 * 104857600}> = emballoc::Allocator::new();

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {

    // search_via_nt_api();
    search_via_volume_winapi();

    0
}