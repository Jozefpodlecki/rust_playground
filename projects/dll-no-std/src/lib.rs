#![no_std]
#![allow(linker_messages)]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "system" fn DllMain(
    _handle: isize,
    _eason: u32,
    _reserved: *mut core::ffi::c_void,
) -> i32 {
    1
}

#[unsafe(no_mangle)]
pub extern "system" fn _DllMainCRTStartup(
    _hinst_dll: *mut core::ffi::c_void,
    _call_reason: u32,
    _reserved: *mut core::ffi::c_void,
) -> u32 {
    1
}