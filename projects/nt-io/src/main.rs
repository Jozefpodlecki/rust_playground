#![no_std]
#![no_main]
#![windows_subsystem = "console"]
#![allow(unused)]

use core::panic::PanicInfo;

extern crate builtins;

use toolkit::{U16CStackString, println};

use crate::{fs::{File, remove_file}, io::Read};

mod io;
mod fs;
mod error;

#[cfg(feature = "alloc")]
#[global_allocator]
static ALLOCATOR: emballoc::Allocator<10485760> = emballoc::Allocator::new();

#[cfg(feature = "alloc")]
#[macro_use]
extern crate alloc;

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info:?}");
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {

    let path = r#"\??\C:\repos\rust_playground\projects\win-veh\target\.rustc_info.json"#;
    if let Err(err) = remove_file(path) {
        println!("{}", err);
    }
    
    // let mut file = File::open().unwrap();

    // let mut buffer = vec![];
    // file.read_to_end(&mut buffer).unwrap();

    // let mut out_file = File::create().unwrap();

    // let src = r#"\??\C:\Users\jozef\Downloads\ghidra_12.1.2_PUBLIC_20260605\ghidra_12.1.2_PUBLIC\bom.json"#;
    // let dest = r#"\??\C:\repos\rust_playground\test.json"#;
    // fs::manual_copy(src, dest).unwrap();
    // let metadata = file.metadata().unwrap();
    // println!("{}", metadata);
    
    0
}