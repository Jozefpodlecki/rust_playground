#![no_std]
#![no_main]
#![windows_subsystem = "console"]
#![allow(unused)]

use alloc::boxed::Box;
use ntapi::{ntpebteb::TEB, winapi_local::um::winnt::NtCurrentTeb};
use toolkit::println;
use winapi::shared::ntdef::PVOID;

use crate::allocator::ThreadLocalAllocator;

extern crate builtins;

#[macro_use]
extern crate alloc;

mod allocator;

#[global_allocator]
static ALLOCATOR: emballoc::Allocator<8192> = emballoc::Allocator::new();

#[inline(never)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{info:?}");
    loop {}
}

#[repr(C)]
pub struct ThreadLocalData {
    pub counter: u32,
    pub woken: bool,
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {
    let data = ThreadLocalData {
        counter: 1,
        woken: false
    };
    let slot = ThreadLocalAllocator::alloc(data).unwrap();

    let data = ThreadLocalAllocator::get_mut::<ThreadLocalData>(slot).unwrap();
    data.counter += 1;

    let data = ThreadLocalAllocator::take::<ThreadLocalData>(slot).unwrap();

    println!("{}", data.counter);

    0
}