#![no_std]
#![no_main]
#![windows_subsystem = "console"]
#![allow(unused)]

use core::panic::PanicInfo;

use toolkit::{Sleeper, U16CStackString, println} ;

use crate::builder::{InjectionBuilder, InjectionError} ;

#[macro_use]
extern crate alloc;

extern crate builtins;

#[global_allocator]
static ALLOCATOR: emballoc::Allocator<4194304> = emballoc::Allocator::new();

mod builder;
mod shellcode;
mod encoder;
mod injector;
mod utils;

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info:?}");
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {

    match run() {
        Ok(_) => 0,
        Err(err) => {
            println!("{:?}", err);
            err.into()
        },
    }
}

fn run() -> Result<(), InjectionError> {
    let file_path = U16CStackString::<200>::from_str(
        r#"C:\repos\rust_playground\projects\exe-no-std\target\debug\exe-no-std.exe"#
    ).unwrap();

    
    InjectionBuilder::new(file_path).build()?;
    Sleeper::sleep(5000);

    Ok(())
}