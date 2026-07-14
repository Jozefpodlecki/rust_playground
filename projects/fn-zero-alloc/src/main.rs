#![no_std]
#![no_main]
#![windows_subsystem = "console"]
#![feature(unsafe_cell_access)]
#![feature(generic_atomic)]
#![allow(unused)]
#![allow(static_mut_refs)]

extern crate builtins;

use core::any::TypeId;
use core::marker::PhantomData;
use core::panic::PanicInfo;
use core::ptr;
use core::mem;

use alloc::string::String;
use toolkit::println;
use winapi::um::winnt::STATUS_INVALID_PARAMETER;

mod types;
mod arena;
mod handle;

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    loop {}
}

#[macro_use]
extern crate alloc;

#[global_allocator]
static ALLOCATOR: emballoc::Allocator<1024> = emballoc::Allocator::new();

zero_alloc_arena!(4096);

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {
    
    let test = [0; 100];

    loop {
        let handle = FunctionAllocator::store(move || {
            test.len()
        });

        if handle.is_none() {
            break;
        }
    }

    FunctionAllocator::debug_dump();

    println!("FunctionAllocator::count {}", FunctionAllocator::count());
    // for handle in FunctionAllocator::iter() {
    //     println!("data_size 0x{:X}", handle.data_size());
    //     let result: i32 = handle.call();
    //     println!("Iter result: {}", result);
    // }

    // let stacked = FunctionAllocator::remove::<500>(handle).unwrap();
    // let result: i32 = stacked.call();
    // println!("Stacked result: {}", result);

    0
}