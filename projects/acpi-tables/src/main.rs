#![no_std]
#![no_main]
#![windows_subsystem = "console"]

use core::mem::zeroed;
use core::panic::PanicInfo;
use core::ptr::null_mut;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::{format, vec};
use ntapi::ntexapi::{NtQuerySystemInformationEx, SYSTEM_FIRMWARE_TABLE_INFORMATION, SystemFirmwareTableEnumerate, SystemFirmwareTableGet, SystemFirmwareTableInformation};
use toolkit::syscalls::NtQuerySystemInformation;
use toolkit::{FreeListAllocator1KB, println};
use winapi::shared::ntstatus::STATUS_UNSUCCESSFUL;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::sysinfoapi::{EnumSystemFirmwareTables, GetSystemFirmwareTable};
use winapi::um::winnt::PVOID;

use crate::acpi::{AcpiEnumIter, walk_acpi_blob};
use crate::parse::iter;
use crate::types::{DmiIter};

mod types;
mod acpi;
mod parse;
mod enums;

extern crate builtins;
extern crate alloc;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    loop {}
}

fn last_error() -> u32 {
    unsafe { GetLastError() }
}

#[global_allocator]
static ALLOCATOR: FreeListAllocator1KB = FreeListAllocator1KB::new();

#[repr(C, packed)]
pub struct RawSMBIOSData {
    pub Used20CallingMethod: u8,
    pub SMBIOSMajorVersion: u8,
    pub SMBIOSMinorVersion: u8,
    pub DmiRevision: u8,
    pub Length: u32,
    pub SMBIOSTableData: [u8; 1],
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {
    let Some(iter) = AcpiEnumIter::new() else {
        println!("enumeration failed");
        return 1;
    };

    let provider = u32::from_be_bytes(*b"ACPI");

    for sig in iter {
        let mut buffer = [0u8; 5000];
        let info_ptr = buffer.as_mut_ptr() as *mut SYSTEM_FIRMWARE_TABLE_INFORMATION;
        let info = unsafe { &mut *info_ptr };
        info.Action = SystemFirmwareTableGet;
        info.ProviderSignature = provider;
        info.TableID = sig.as_u32();

        let mut expected = 0;
        let status = unsafe {
            NtQuerySystemInformation(
                SystemFirmwareTableInformation,
                buffer.as_mut_ptr() as *mut _,
                buffer.len() as u32,
                &mut expected,
            )
        };

        if status != 0 {
            println!("{}: failed 0x{:X}", sig, status);
            continue;
        }

        let len = info.TableBufferLength as usize;
        let table = unsafe {
            core::slice::from_raw_parts(info.TableBuffer.as_ptr(), len)
        };

        println!("{} ({} bytes)", sig, len);
    }

    0
}

pub fn parse_smbios() {
     // let mut buffer: SYSTEM_FIRMWARE_TABLE_INFORMATION = unsafe { zeroed() };
    let mut buffer = [0; 5000];
    let info_ptr = unsafe { &mut buffer as *mut _ as *mut SYSTEM_FIRMWARE_TABLE_INFORMATION };
    let info = { unsafe { &mut *info_ptr } };
    info.Action = SystemFirmwareTableGet;
    info.ProviderSignature = u32::from_be_bytes(*b"RSMB");
    let mut expected = 0;

    let status = unsafe { NtQuerySystemInformation(
        SystemFirmwareTableInformation,
        &mut buffer as *mut _ as *mut _,
        buffer.len() as u32,
        &mut expected) };

        // STATUS_UNSUCCESSFUL
    // 
    let smbios = unsafe { &*(info.TableBuffer.as_ptr() as *const RawSMBIOSData) };
    let length = smbios.Length;

    let table = unsafe {
        core::slice::from_raw_parts(
            smbios.SMBIOSTableData.as_ptr(),
            length as usize,
        )
    };

    for s in iter(table) {
        println!("{}", s);
    }

    // if let Some(uuid) = read_uuid(table) {
    //     println!("UUID: {}", uuid_fmt(&uuid));
    // }
}