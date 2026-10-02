#![no_std]
#![no_main]
#![windows_subsystem = "console"]

#[inline(never)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    win_platform::default_panic_handler(info)
}

extern crate compiler_builtins;

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {

    0
}

