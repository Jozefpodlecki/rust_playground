#![no_std]
#![no_main]
#![windows_subsystem = "console"]

use core::{panic::PanicInfo, ptr::null_mut};

use toolkit::{U16CStackString, canonicalize, println};

extern crate builtins;

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {
    let path = U16CStackString::<50>::from_str("test.exe").unwrap();
    let path = canonicalize::<_, 100>(path).unwrap();
    println!("{}", path);
    
    0
}