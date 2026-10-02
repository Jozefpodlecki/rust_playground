#![no_std]
#![allow(unused, unsafe_op_in_unsafe_fn)]
#![feature(const_atomic)]
#![feature(arbitrary_self_types_pointers)]

mod free_list;

pub use free_list::*;

pub type FreeListAllocator1KB = FreeListAllocator<1024>;
pub type FreeListAllocator1MB = FreeListAllocator<{1024 * 1024}>;
pub type FreeListAllocator10MB = FreeListAllocator<{10 * 1024 * 1024}>; 
pub type FreeListAllocator100MB = FreeListAllocator<{100 * 1024 * 1024}>; 