#![no_std]
#![no_main]
#![windows_subsystem = "console"]

mod extractor;
mod types;
mod dump;

use core::panic::PanicInfo;

use crate::{dump::dump_all, extractor::CpuidExtractor};

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {
    let cpu_info = CpuidExtractor::extract();
    dump_all(&cpu_info);
    0
}