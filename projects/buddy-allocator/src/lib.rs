#![cfg_attr(not(test), no_std)]
#![feature(sync_unsafe_cell)]
#![allow(static_mut_refs, non_snake_case, non_camel_case_types)]
#![feature(arbitrary_self_types_pointers)]
#![feature(ptr_alignment_type)]
#![feature(allocator_api)]
#![allow(unused)]

pub mod types;
pub mod buddy;
pub mod allocator;
pub mod stress;

pub use allocator::*;