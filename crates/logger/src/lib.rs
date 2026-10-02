#![no_std]
#![feature(core_io)]
#![allow(static_mut_refs)]
#![feature(sync_unsafe_cell)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod error;
mod file;
mod console;
mod agg;
mod builder;

pub use error::*;
pub use file::*;
pub use console::*;
pub use builder::*;

use core::fmt;

pub trait WinLogger: log::Log + Send + Sync + 'static {
    fn log_fmt(&self, args: fmt::Arguments<'_>) -> Result<(), LoggerError>;
    fn flush(&self) -> Result<(), LoggerError>;
}