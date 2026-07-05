#![no_std]
#![no_main]
#![windows_subsystem = "console"]
#![feature(arbitrary_self_types_pointers)]
#![feature(sync_unsafe_cell)]
#![feature(ptr_alignment_type)]
#![feature(rustc_attrs, core_intrinsics)]
#![allow(unsafe_op_in_unsafe_fn)]

use core::{arch::{asm, naked_asm}, mem, panic::PanicInfo, ptr::{self, null_mut}, sync::atomic::{AtomicUsize, Ordering}};

use alloc::string::String;
use toolkit::println;

extern crate builtins;

#[macro_use]
extern crate alloc;

mod free_text_alloc;
mod arena;
mod types;

pub use free_text_alloc::*;

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

fn main() {
    let mut value = String::from("test");
    value += "abc";
    println!("{}", value);
    println!("{:p}", value.as_ptr());

    let mut value = String::from("abc");
    value += "test";
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {

    init_arena();

    main();

    unsafe { 
        asm!(
            "xor rax, rax",
            "add rsp, 0x28",
            "ret"
        );
        core::intrinsics::unreachable();
        #[allow(unreachable_code)]
        arena::arena_memory();
    }

    0
}