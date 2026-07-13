#![no_std]
#![no_main]
#![windows_subsystem = "console"]

use core::{arch::naked_asm, panic::PanicInfo};

use toolkit::{ProcessEnvironmentBlock, println};

extern crate builtins;

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {

    let peb = ProcessEnvironmentBlock::current_process();
    
    // println!("{}", peb);
    println!("{}", peb.executable_path().file_name());
    println!("{}", peb.executable_path().file_stem());
    println!("{}", peb.executable_path().extension());
    println!("{}", peb.executable_path().parent());
    println!("{}", peb.executable_path().parent().parent());
    println!("{}", peb.executable_path().directory_name());

    0
}