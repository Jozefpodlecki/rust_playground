#![no_std]
#![feature(sync_unsafe_cell)]
#![feature(naked_functions_rustic_abi)]
#![feature(core_intrinsics)]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(static_mut_refs)]
#![allow(non_snake_case)]
#![allow(internal_features)]
#![allow(unused)]
#![allow(incomplete_features)]
#![feature(arbitrary_self_types_pointers)]
#![feature(generic_const_exprs)]
#![feature(unsize)]
#![feature(allocator_api)]
#![feature(slice_ptr_get)]
#![feature(generic_atomic)]
#![feature(utf16_extra)]
#![feature(core_io)]
#![feature(core_io_internals)]
#![feature(io_const_error)]
#![feature(const_trait_impl)]
#![feature(const_slice_make_iter)]
#![feature(str_internals)]
#![feature(const_index)]
#![feature(const_cmp)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod types;
pub mod utils;
mod stubs;
mod status;
mod peb;
mod kuser;

#[cfg(feature = "alloc")]
pub mod fs;

pub mod io;
pub mod ntdll;
pub mod syscalls;

pub use stubs::*;
pub use status::*;
pub use peb::*;
pub use kuser::*;