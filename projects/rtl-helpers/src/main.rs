#![no_std]
#![no_main]
#![windows_subsystem = "console"]
#![allow(unused)]
#![allow(static_mut_refs)]

use core::{arch::naked_asm, panic::PanicInfo, ptr::null_mut};

use ntapi::ntrtl::{RtlCreateProcessParametersEx, RtlGetExePath, RtlGetNtSystemRoot};
use toolkit::{ProcessEnvironmentBlock, U16CStackString, println};

use crate::{fiber::FiberScheduler, flags::ProcessParametersFlags, string::UnicodeString};

extern crate builtins;

mod examples;
mod fiber;
mod flags;
mod string;
mod memory;

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
     println!("{info}");
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {
    if let Some(mut scheduler) = FiberScheduler::new() {
        scheduler.run();
    }

    0
}