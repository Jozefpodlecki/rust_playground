#![no_std]
#![no_main]
#![windows_subsystem = "console"]
#![allow(static_mut_refs)]
#![feature(sync_unsafe_cell)]
#![allow(unused)]

 
use core::{mem, panic::PanicInfo};

use hashbrown::HashSet;
use toolkit::{U16CStackString, println};
use winapi::{shared::{guiddef::GUID, minwindef::{DWORD, ULONG, USHORT}, ntdef::{BOOLEAN, HANDLE, PVOID}}, um::{errhandlingapi::GetLastError, fileapi::{CreateFileW, OPEN_EXISTING, ReadFile}, winbase::OpenFile, winnt::{FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_ALL, GENERIC_READ, PROCESS_QUERY_LIMITED_INFORMATION}}};

use crate::{dev_inf::HidDeviceIterator, modules::SystemModuleIterator, object_type::ObjectTypeIterator, types::file_name, utils::*};

extern crate builtins;

#[macro_use]
extern crate alloc;

#[global_allocator]
static ALLOCATOR: emballoc::Allocator<10485760> = emballoc::Allocator::new();

mod error;
mod modules;
mod utils;
mod handle;
mod object_type;
mod dev_inf;
mod types;

system_handle_iterator!(8_000_000);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HIDD_ATTRIBUTES {
    pub Size: ULONG,
    pub VendorID: USHORT,
    pub ProductID: USHORT,
    pub VersionNumber: USHORT,
}

pub type PHIDD_ATTRIBUTES = *mut HIDD_ATTRIBUTES;

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    loop {}
} 

#[link(name = "hid")]
unsafe extern {
    pub fn HidD_GetHidGuid(HidGuid: *mut GUID);
    pub fn HidD_GetAttributes(HidDeviceObject: HANDLE, Attributes: PHIDD_ATTRIBUTES) -> BOOLEAN;
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {

    // let iter = ObjectTypeIterator::new().unwrap();

    // for obj in iter {
    //     println!("{}: {}", obj.id, obj.name());
    // }
    // return 0;

    let iter = HidDeviceIterator::all().unwrap();

    for entry in iter {
        println!("{:?} {:?} {:?} {:?}", entry.device_info_data.DevInst, entry.friendly_name(), entry.description(), entry.get_device_path());
    }

    // let mut hid_guid = unsafe { core::mem::zeroed::<GUID>() };
    // unsafe {
    //     HidD_GetHidGuid(&mut hid_guid);
    // }
    // println!("{:?}", hid_guid);
    // let iter = HidDeviceIterator::new(hid_guid).unwrap();

    // for entry in iter {
    //     println!("{:?} {:?} {:?} {}", entry.device_info_data.DevInst, entry.friendly_name(), entry.description(), entry.get_device_path().unwrap());
    // }

    // let file_name = U16CStackString::<200>::from_str(r#"\\?\hid#vid_09da&pid_8736#8&f0f3bb&0&0000#{4d1e55b2-f16f-11cf-88cb-001111000030}"#).unwrap();
    // let handle = unsafe { 
    //     CreateFileW(
    //         file_name.as_ptr(),
    //         0,
    //         FILE_SHARE_READ | FILE_SHARE_WRITE,
    //         core::ptr::null_mut(),
    //         OPEN_EXISTING,
    //         0,
    //         core::ptr::null_mut(),
    //     )
    //  };
    // println!("CreateFileW {:p}", handle);

    // let mut buffer = [0u8; 64];  // HID reports are typically 64 bytes or less
    // let mut bytes_read: DWORD = 0;

    // let mut attributes: HIDD_ATTRIBUTES = unsafe { mem::zeroed() };
    // attributes.Size = mem::size_of::<HIDD_ATTRIBUTES>() as u32;

    // let result = unsafe { HidD_GetAttributes(handle, &mut attributes) };
    // if result != 0 {
    //     println!("Vendor ID: 0x{:04X}", attributes.VendorID);
    //     println!("Product ID: 0x{:04X}", attributes.ProductID);
    //     println!("Version: 0x{:04X}", attributes.VersionNumber);
    // }

    // loop {
    //     let result = unsafe {
    //         ReadFile(
    //             handle,
    //             buffer.as_mut_ptr() as *mut _,
    //             buffer.len() as u32,
    //             &mut bytes_read,
    //             core::ptr::null_mut(),  // No OVERLAPPED structure
    //         )
    //     };

    //     if result == 0 {
    //         let error = unsafe { GetLastError() };
    //         println!("ReadFile error {error}");
    //         if error == 997 {  // ERROR_IO_PENDING
    //             // Handle async if using overlapped I/O
    //             continue;
    //         }
    //         break;
    //     }

    //     if bytes_read > 0 {
    //         println!("Read {} bytes: {:02X?}", bytes_read, &buffer[..bytes_read as usize]);
    //         // Process the HID report
    //     }
    // }

    // 

    // let iter = SystemModuleIterator::new().unwrap();

    // for entry in iter {
    //     println!("{}", entry.file_path().full_path());

    //     if entry.file_path().full_path().contains("mouhid.sys") {
    //         break;
    //     }
    // }

    // let file_object_type = 42;
    // let iter = SystemHandleIterator::new().unwrap();
    // let mut map = HashSet::new();

    // for handle in iter.filter(|pr| pr.object_type_index() == file_object_type) {

    //     match open_process(handle.process_id(), PROCESS_QUERY_LIMITED_INFORMATION) {
    //         Some(target_process) => {
    //             match file_name(target_process) {
    //                 Ok(file_name) => {
    //                     map.insert(file_name);
    //                     unsafe { NtClose(target_process); }
    //                 },
    //                 Err(_) => continue,
    //             }
    //         }
    //         None => continue,
    //     }
    // }

    // for value in map {
    //     println!("{}", value.as_str());
    // }

    0
}
