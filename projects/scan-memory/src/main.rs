#![no_std]
#![no_main]
#![windows_subsystem = "console"]

use core::panic::PanicInfo;

use ntapi::ntpsapi::NtCurrentProcess;
use toolkit::{MemoryRegionIterator, MemoryRegionReverseIterator, Sleeper, maximum_user_address, println};

extern crate builtins;

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {

    println!("maximum_user_address 0x{:X}", maximum_user_address());
    // let iter = MemoryRegionReverseIterator::from_max_address(NtCurrentProcess);

    // for region in iter {
    //     println!("{region}");
    //     Sleeper::sleep(100);
    // }

    let iter = MemoryRegionIterator::new(NtCurrentProcess);

    for region in iter {
        println!("{region}");
        Sleeper::sleep(100);
    }

    0
}