#![no_std]
#![no_main]
#![allow(static_mut_refs, non_snake_case, non_camel_case_types)]
#![windows_subsystem = "console"]
#![feature(arbitrary_self_types_pointers)]
#![feature(sync_unsafe_cell)]
// #![feature(allocator_api)]
// #![feature(naked_functions_rustic_abi)]
#![feature(ptr_alignment_type)]
#![feature(rustc_attrs)]
#![allow(unused)]

use core::ops::AddAssign as _;

use alloc::string::String;
use toolkit::println;

#[macro_use]
extern crate alloc;

extern crate builtins;

#[inline(never)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{info}");
    loop {}
}

mod allocator;
mod rustc_alloc;

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {

    init_arena!(4096);

    let mut value = String::from("a");
    
    loop {
        // value += "a";
        value.add_assign("a");

        if value.capacity() > 512 {
            let test = crate::allocator::FreeListAllocator::get();

            for block in test.free_blocks() {
                println!("{}", block);
            }

            return 0
        }

        // Sleeper::sleep(1000);
    }

    0
}
