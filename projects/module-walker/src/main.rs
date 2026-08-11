#![no_std]
#![no_main]
#![windows_subsystem = "console"]
#![feature(str_from_raw_parts)]

extern crate builtins;

use core::{arch::naked_asm, panic::PanicInfo, ptr::null_mut};

use ntapi::{ntexapi::{MutantBasicInformation, NtQueryMutant}, ntpebteb::PEB, ntpsapi::NtCurrentProcess};
use toolkit::{NtDll, ProcessEnvironmentBlock, ProcessMemoryReader, println};

use crate::{ldr::LdrModuleIterator, nt_ldr::NtModuleIterator};

mod ldr;
mod nt_ldr;
mod exports;

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {

    let peb = ProcessEnvironmentBlock::current_process();
    let ntdll = NtDll::from_current_process();

    // 
    let modules = NtModuleIterator::new(NtCurrentProcess);

    for (index, module) in modules.enumerate() {
        // println!("{:p} {}", module.base_address(), module.file_name());
        if module.file_name() == "ntdll.dll" {
            println!("Base: {:p}", module.base_address());
            println!("Size: {:#x}", module.size_of_image());
            
            // if let Some(exports) = module.exports() {
            //     for export in exports {
            //         println!("  {} ({})", export.name, export.address);


            //     }
            // }
        }
    }

    let modules = LdrModuleIterator::new(peb);

    for module in modules {
        // println!("{}, 0x{:X}", module.file_name, module.base_address());

        if module.file_name == "ntdll.dll" {
            println!("Base: {:#x}", module.base_address());
            println!("Size: {:#x}", module.size_of_image());
            
            // if let Some(exports) = module.exports() {
            //     for export in exports {
            //         println!("  {} ({})", export.name, export.address);


            //     }
            // }
        }
    }

    

    0
}

// pub fn entrypoint(peb_ptr: *const c_void) -> Result<*mut c_void, NTSTATUS> {
//     let handle = NtCurrentProcess;
//     let peb: PEB = ProcessMemoryReader::read_remote(handle, peb_ptr as _)?;
//     let image_base = peb.ImageBaseAddress;

//     let dos_header: IMAGE_DOS_HEADER = 
//         ProcessMemoryReader::read_remote(handle, image_base)?;

//     let nt_headers_addr = (image_base as usize + dos_header.e_lfanew as usize) as *const c_void;
//     let nt_headers: IMAGE_NT_HEADERS64 = 
//         ProcessMemoryReader::read_remote(handle, nt_headers_addr as _)?;

    
// }


