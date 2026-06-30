#![no_std]
#![no_main]
#![windows_subsystem = "console"]
#![allow(static_mut_refs)]

mod runtime;
mod task;
mod queue;
mod executor;
mod channel;
mod timer;
mod future;
mod fs;
mod io;

use core::time::Duration;

use utils::*;

use crate::{future::Join, runtime::spawn, timer::Delay};

#[macro_use]
extern crate alloc;

#[global_allocator]
static ALLOCATOR: emballoc::Allocator<8192> = emballoc::Allocator::new();

#[inline(never)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{info:?}");
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {
    let mut runtime = runtime::Runtime::new();
    
    runtime.block_on(async {
        Delay::new(Duration::from_secs(1)).await;

        let result = spawn(async {
            42
        }).await;

        let task1 = spawn(async {
            Delay::new(Duration::from_secs(3)).await;
            111
        });

        let task2 = spawn(async {
            420
        });

        let results = Join::new(task1, task2).await;
        println!("{result} {results:?}");
    });

    0
}