#![no_std]
#![no_main]
#![windows_subsystem = "console"]
#![allow(static_mut_refs)]
#![feature(sync_unsafe_cell)]

use core::{mem, panic::PanicInfo, ptr, slice};

use toolkit::{ProcessMemoryReader, ProcessQuerier, U8CStackString, println};
use winapi::um::handleapi::CloseHandle ;

use crate::{handle::SystemHandleIterator, object_type::ObjectTypeIterator, utils::*};

extern crate builtins;

#[macro_use]
extern crate alloc;

#[global_allocator]
static ALLOCATOR: emballoc::Allocator<10485760> = emballoc::Allocator::new();

mod error;
mod utils;
mod handle;
mod object_type;

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {

    // let iter = ObjectTypeIterator::new().unwrap();

    // for obj in iter {
    //     println!("{}: {}", obj.id, obj.name());
    // }
    // return 0;

    let file_object_type = 42;
    let iter = SystemHandleIterator::new().unwrap();

    let process_name = U8CStackString::<20>::from_str("exe-no-std.exe").unwrap();
    let process_info = ProcessQuerier::find_process_by_name(&process_name);

    let process_info = match process_info {
        Some(value) => value,
        None => {
            println!("Not found");
            return 0
        },
    };

    for handle in iter.filter(|pr| pr.object_type_index() == file_object_type) {

        if handle.process_id() != process_info {
            continue;
        }
   
        let duped_handle = match duplicate_handle(&handle) {
            Some(value) => value,
            None => {
                continue;
            },
        };

        if let Some(process_name) = ProcessQuerier::get_process_name_by_pid::<100>(handle.process_id()) {
            println!("Process({}): {}", handle.process_id(), process_name);
        }

        println!("handle_attributes={}, granted_access={}, handle_value=0x{:X}, object={}, object_type_index={}",
            handle.handle_attributes(),
            handle.granted_access(),
            handle.handle_value(),
            handle.object(),
            handle.object_type_index());
            //STATUS_INVALID_DEVICE_REQUEST: NTSTATUS = 0xC0000010;
            // STATUS_OBJECT_TYPE_MISMATCH: NTSTATUS = 0xC0000024;
            // println!("is_directory_handle={}", is_directory_handle(duped_handle as _));


        if let Some(file_name) = get_full_path_manual(duped_handle) {
            println!("File: {}", file_name.as_str());
        }
        
        unsafe { CloseHandle(duped_handle); }
    }

    0
}