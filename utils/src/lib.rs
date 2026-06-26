#![no_std]
#![feature(sync_unsafe_cell)]
#![feature(naked_functions_rustic_abi)]
#![allow(static_mut_refs)]

mod crt;
mod nt_console;
mod u16_stack_string;
mod u8_stack_string;
mod helpers;
mod memory;
mod ntdll;


#[cfg(feature = "alloc")]
mod thread;

#[cfg(feature = "alloc")]
mod arc;

pub use crt::*;
pub use nt_console::*;
pub use u16_stack_string::*;
pub use u8_stack_string::*;
pub use helpers::*;
pub use memory::*;
pub use ntdll::*;

#[cfg(feature = "alloc")]
pub use thread::*;

#[cfg(feature = "alloc")]
pub use arc::*;