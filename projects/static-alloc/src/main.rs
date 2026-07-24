// #![no_std]
// #![no_main]
#![allow(static_mut_refs, non_snake_case, non_camel_case_types)]
#![windows_subsystem = "console"]
#![feature(arbitrary_self_types_pointers)]
#![feature(sync_unsafe_cell)]
// #![feature(allocator_api)]
// #![feature(naked_functions_rustic_abi)]
#![feature(ptr_alignment_type)]
#![feature(rustc_attrs)]

#[macro_use]
extern crate alloc;

mod static_alloc;

extern crate builtins;

use crate::static_alloc::{ARENA, FreeListAllocator};

// #[inline(never)]
// #[panic_handler]
// fn panic(_info: &core::panic::PanicInfo) -> ! {
//     loop {}
// }

// #[global_allocator]
static ALLOCATOR: FreeListAllocator = FreeListAllocator;

#[rustc_std_internal_symbol]
#[rustc_allocator]
fn __rust_alloc(size: usize, align: ::core::mem::Alignment) -> *mut u8 {
    unsafe { ::core::alloc::GlobalAlloc::alloc(
        &ALLOCATOR,
        ::core::alloc::Layout::from_size_alignment_unchecked(size, align),
    ) }
}

#[rustc_std_internal_symbol]
#[rustc_deallocator]
fn __rust_dealloc(
    ptr: *mut u8,
    size: usize,
    align: ::core::mem::Alignment,
) -> () {
    unsafe { ::core::alloc::GlobalAlloc::dealloc(
        &ALLOCATOR,
        ptr,
        ::core::alloc::Layout::from_size_alignment_unchecked(size, align),
    ) }
}

#[rustc_std_internal_symbol]
#[rustc_reallocator]
fn __rust_realloc(
    ptr: *mut u8,
    size: usize,
    align: ::core::mem::Alignment,
    new_size: usize,
) -> *mut u8 {
    unsafe { ::core::alloc::GlobalAlloc::realloc(
        &ALLOCATOR,
        ptr,
        ::core::alloc::Layout::from_size_alignment_unchecked(size, align),
        new_size,
    ) }
}

#[rustc_std_internal_symbol]
#[rustc_allocator_zeroed]
fn __rust_alloc_zeroed(
    size: usize,
    align: ::core::mem::Alignment,
) -> *mut u8 {
    unsafe {::core::alloc::GlobalAlloc::alloc_zeroed(
        &ALLOCATOR,
        ::core::alloc::Layout::from_size_alignment_unchecked(size, align),
    ) }
}

// #[unsafe(no_mangle)]
// pub extern "C" fn mainCRTStartup() -> i32 {

//     let mut test = vec![0; 1024 * 128];
//     // let mut test = vec![0; 1024 * 512];
//     let used = unsafe { (*ARENA.get()).used() };
//     used as i32
// }

fn main() {
    let mut test = vec![0; 1024 * 128];
    // let mut test = vec![0; 1024];
    // let mut test = vec![0; 1024];
    // let mut test = vec![0; 1024];
    // let mut test = vec![0; 1024 * 512];
    let used = unsafe { (*ARENA.get()).used() };
    println!("{}", used);
}