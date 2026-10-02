#![no_std]
#![no_main]
#![windows_subsystem = "console"]
#![allow(unused)]
#![allow(static_mut_refs)]
#![feature(sync_unsafe_cell)]

use allocator::FreeListAllocator1KB;
use win_platform::STATUS_FATAL_APP_EXIT;

extern crate alloc;
extern crate compiler_builtins;

#[global_allocator]
static ALLOCATOR: FreeListAllocator1KB = FreeListAllocator1KB::new();

mod types;
mod constants;
mod error;
mod event;
mod async_server;
mod server;
mod async_client;
mod client;
mod setup;
mod utils;

#[inline(never)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    win_platform::default_panic_handler(info)
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {

    match setup::setup() {
        Ok(_) => 0,
        Err(err) => {
            log::error!("{err}");
            STATUS_FATAL_APP_EXIT
        },
    }
}