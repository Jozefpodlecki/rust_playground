#![no_std]
#![no_main]
#![windows_subsystem = "console"]

use toolkit::{File, U16CStackString, Write, canonicalize, println, read};

#[macro_use]
extern crate alloc;

#[global_allocator]
static ALLOCATOR: emballoc::Allocator<10485760> = emballoc::Allocator::new();

use crate::{types::*};

extern crate builtins;

mod types;
mod utils;

#[inline(never)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{:?}", info);
    loop {}
}


#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {

    // let path = U16CStackString::<80>::from_str(r"\??\C:\repos\rust_playground\projects\text-alloc\target\debug\text-alloc.exe").unwrap();
    let path = U16CStackString::<80>::from_str(r"\??\C:\repos\rust_playground\projects\exe-no-std\target\release\exe-no-std.exe").unwrap();
    let buffer = read::<_, 4096>(path).unwrap();

    {
        
        let pefile = pelite::pe64::PeFile::from_bytes(&buffer).unwrap();    
        let options = SerializedBinaryOptions::all();
        let binary = Binary::new(pefile, &options).unwrap();
        let path = U16CStackString::<80>::from_utf8_bytes(br#"exe-no-std.json"#).unwrap();
        let path = canonicalize::<_, 100>(path).unwrap();
        let mut file = File::create(path).unwrap();

        let mut buffer = serde_json::to_vec_pretty(&binary).unwrap();
        file.write_all(&mut buffer).unwrap();
    }

    0
}