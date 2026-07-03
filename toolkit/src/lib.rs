#![no_std]
#![feature(sync_unsafe_cell)]
#![feature(naked_functions_rustic_abi)]
#![feature(core_intrinsics)]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(static_mut_refs)]
#![allow(non_snake_case)]
#![allow(unused, internal_features)]
#![feature(arbitrary_self_types_pointers)]
#![feature(generic_const_exprs)]

mod fs;
mod io;
mod error;
pub mod syscalls;
mod nt_console;
mod u16_stack_string;
mod u8_stack_string;
mod helpers;
mod memory;
mod ntdll;
mod mutex;
mod static_vec;
pub mod types;

pub use fs::*;
pub use io::*;
pub use error::*;
pub use nt_console::*;
pub use u16_stack_string::*;
pub use u8_stack_string::*;
pub use helpers::*;
pub use memory::*;
pub use mutex::*;
pub use ntdll::*;

#[cfg(feature = "alloc")]
mod thread;

#[cfg(feature = "alloc")]
mod arc;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
pub use thread::*;

#[cfg(feature = "alloc")]
pub use arc::*;