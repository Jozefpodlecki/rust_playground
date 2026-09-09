#![no_std]
#![no_main]
#![windows_subsystem = "console"]

use core::panic::PanicInfo;

use toolkit::{FreeListAllocator1KB, ProcessSpawner, Sleeper, U16CStackString, println};

use crate::version::*;

#[macro_export]
extern crate alloc;

mod version;

extern crate builtins;

#[global_allocator]
static ALLOCATOR: FreeListAllocator1KB = FreeListAllocator1KB::new();

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {
    
    let info = SystemVersionInformation::new();
    println!("{}", info);

    let product_info = ProductInfo::new();
    println!("{product_info:?}");

    let version_info = VersionInfo::new();
    println!("{version_info}");

    let appserver_mode = check_license_appserver_mode().unwrap();
    println!("AppServer Mode: {}", appserver_mode);

    0
}