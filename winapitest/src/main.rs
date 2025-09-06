use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt, ptr::null_mut};

use anyhow::Result;
use log::info;
use minidump::*;
use windows::{core::PCWSTR, Win32::{Foundation::HANDLE, Storage::FileSystem::{CreateFileW, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_WRITE, FILE_SHARE_MODE, FILE_SHARE_NONE}, System::{Diagnostics::Debug::{MiniDumpWriteDump, MINIDUMP_TYPE}, Threading::{GetCurrentProcess, GetCurrentProcessId}}}};

fn to_pcwstr(s: &str) -> PCWSTR {
    let wide: Vec<u16> = OsStr::new(s).encode_wide().chain(once(0)).collect();
    PCWSTR(wide.as_ptr())
}

fn main() -> Result<()> {

    let output_path = std::env::current_dir()?;
    let file_path = output_path.join("test.dmp");

    unsafe  {
        // let process_handle = 0;
        // let process_id = 0;
        let process_handle = unsafe { GetCurrentProcess() };
        let process_id = unsafe { GetCurrentProcessId() };

        info!("Handle: {:?} Id: {}", process_handle, process_id);

        let dump_type = MINIDUMP_TYPE(0);

        let file_handle = unsafe {
            CreateFileW(
                to_pcwstr(&file_path.file_name().unwrap().to_string_lossy()),
                FILE_GENERIC_WRITE.0,
                FILE_SHARE_NONE,
                None,
                CREATE_ALWAYS,
                FILE_ATTRIBUTE_NORMAL,
                None
            )
        }?;

        MiniDumpWriteDump(
            process_handle,
            process_id,
            file_handle,
            dump_type,
            None,
            None,
            None
        )?;

    }
    
    let mut dump = minidump::Minidump::read_path(file_path)?;
    if let Ok(system_info) = dump.get_stream::<MinidumpSystemInfo>() {

    }
    
    if let Ok(exception) = dump.get_stream::<MinidumpException>() {

    }

    if let Ok(threads) = dump.get_stream::<MinidumpThreadList>() {
        // Use `Default` to try to make progress when a stream is missing.
        // This is especially natural for MinidumpMemoryList because
        // everything needs to handle memory lookups failing anyway.
        let mem = dump.get_memory().unwrap_or_default();

        for thread in &threads.threads {
            
            if let Some(stack) = thread.stack_memory(&mem) {
                info!("{}", stack.base_address());
            }
            
        }
    }

    Ok(())
}
