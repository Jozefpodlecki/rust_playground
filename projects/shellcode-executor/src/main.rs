#![no_std]
#![no_main]
#![windows_subsystem = "console"]
#![allow(unused)]
#![allow(static_mut_refs)]

use core::{mem, panic::PanicInfo, ptr};

use ntapi::{ntpebteb::PEB, ntrtl::RTL_USER_PROCESS_PARAMETERS};
use toolkit::{ProcessEnvironmentBlock, ProcessMemoryProtector, U16CStackString, println};

use crate::{shellcode::Shellcode, utils::OctaDisplay};

extern crate builtins;

mod utils;
mod encoder;
mod shellcode;

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static mut SHELLCODE: [u8; 1024] = [0; 1024];

#[global_allocator]
static ALLOCATOR: emballoc::Allocator<4194304> = emballoc::Allocator::new();


#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {

    // let peb = ProcessEnvironmentBlock::current_process();

    // println!("{}", mem::offset_of!(PEB, ProcessParameters));
    // println!("{}", mem::offset_of!(RTL_USER_PROCESS_PARAMETERS, StandardOutput));

    // let mut str = U16CStackString::<50>::from_str(r"Hello world").unwrap();
    // let iter = OctaDisplay::new(str.as_slice());

    // for entry in iter {
    //     println!("{}", entry);
    // }

    // println!("StandardOutput: {:p}", peb.process_params().StandardOutput);

    let shellcode = Shellcode::<500>::nt_write_file().unwrap();

    let buffer = shellcode.into_inner();
    
    unsafe {
        ptr::copy_nonoverlapping(buffer.as_ptr(), SHELLCODE.as_mut_ptr(), buffer.len());
    }

    let protector = ProcessMemoryProtector::current();

    unsafe {
        protector.make_writable(SHELLCODE.as_ptr() as _, 1024).unwrap();
        let func: fn() -> i32 = core::mem::transmute(SHELLCODE.as_ptr());
        func();
    }

    0
}