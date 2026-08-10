#![no_std]
#![no_main]
#![windows_subsystem = "console"]

use core::panic::PanicInfo;

use toolkit::{ProcessEnvironmentBlock, print, println};

use crate::{extensions::FixedEnvironmentExtensions, fixed::FixedEnvironment, types::Environment};

extern crate builtins;

mod utils;
mod types;
mod fixed;
mod heap;
mod extensions;

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {

    let peb = ProcessEnvironmentBlock::current_process();

    let enviroment: *const u16 = peb.environment_raw() as *const u16;
    let environment_size: usize = peb.environment_size();

    let mut env = FixedEnvironment::<16384>::from_raw_parts(enviroment, environment_size / 2).unwrap();
    
    let path = env.get(&"Path").unwrap();
    println!("{}", path);
    
    env.remove(&"Path").unwrap();
    env.remove(&"CARGO").unwrap();
    env.remove(&"CARGO_HOME").unwrap();
    env.remove(&"CARGO_MANIFEST_DIR").unwrap();
    env.remove(&"CARGO_MANIFEST_PATH").unwrap();
    env.remove(&"CARGO_PKG_AUTHORS").unwrap();
    env.remove(&"CARGO_PKG_DESCRIPTION").unwrap();
    env.remove(&"CARGO_PKG_HOMEPAGE").unwrap();
    env.remove(&"CARGO_PKG_LICENSE").unwrap();
    env.remove(&"CARGO_PKG_LICENSE_FILE").unwrap();
    env.remove(&"CARGO_PKG_NAME").unwrap();
    env.remove(&"CARGO_PKG_README").unwrap();
    env.remove(&"CARGO_PKG_REPOSITORY").unwrap();
    env.remove(&"CARGO_PKG_RUST_VERSION").unwrap();
    env.remove(&"CARGO_PKG_VERSION").unwrap();
    env.remove(&"CARGO_PKG_VERSION_MAJOR").unwrap();
    env.remove(&"CARGO_PKG_VERSION_MINOR").unwrap();
    env.remove(&"CARGO_PKG_VERSION_PATCH").unwrap();
    env.remove(&"CARGO_PKG_VERSION_PRE").unwrap();
    
    println!("actual_capacity {}", env.actual_capacity());
    let env = env.shrink::<8196>().unwrap();

    // for (key, value) in env.iter() {
    //     println!("{}={}", key, value);
    // }

    let heap_env = env.to_heap().unwrap();
    println!("heap_env capacity {}", heap_env.capacity());

    for (key, value) in heap_env.iter() {
        println!("{}={}", key, value);
    }

    0
}
