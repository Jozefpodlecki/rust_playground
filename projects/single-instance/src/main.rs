#![no_std]
#![no_main]
#![windows_subsystem = "console"]
#![feature(generic_const_exprs)]
#![allow(static_mut_refs)]



use core::panic::PanicInfo;

use toolkit::println;

mod single_instance;

extern crate builtins;

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

single_instance!(b"SINGLE-INSTANCE");

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {

    if SingleInstance::setup_and_check() {
        println!("test");
    }

     if SingleInstance::setup_and_check() {
        println!("test");
    }

    0
}