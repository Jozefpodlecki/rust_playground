#![no_std]
#![feature(sync_unsafe_cell)]
#![feature(naked_functions_rustic_abi)]
#![allow(static_mut_refs)]

mod crt;
mod nt_console;
mod u16_stack_string;
mod helpers;

pub use crt::*;
pub use nt_console::*;
pub use u16_stack_string::*;
pub use helpers::*;