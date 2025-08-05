use windows::Win32::System::ProcessStatus::{EnumProcessModulesEx, GetModuleInformation, GetModuleFileNameExW, LIST_MODULES_ALL, MODULEINFO};
use windows::Win32::Foundation::{HANDLE, HMODULE};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::Path;
use anyhow::*;

use crate::process_dumper::types::ProcessModule;

pub unsafe fn enumerate_modules(process_handle: HANDLE) -> Result<Vec<ProcessModule>> {
    let mut module_handles = [HMODULE::default(); 64];
    let mut needed = 0u32;
    let count_of_bytes = (module_handles.len() * std::mem::size_of::<HMODULE>()) as u32;

    EnumProcessModulesEx(
        process_handle,
        module_handles.as_mut_ptr(),
        count_of_bytes,
        &mut needed,
        LIST_MODULES_ALL,
    )?;

    let count = (needed as usize) / std::mem::size_of::<HMODULE>();
    let module_handles = &module_handles[..count];

    let mut modules = Vec::with_capacity(count);
    let module_size = std::mem::size_of::<MODULEINFO>() as u32;

    for &mod_handle in module_handles {
        let mut info = MODULEINFO::default();
        GetModuleInformation(process_handle,
            mod_handle,
            &mut info,
            module_size)?;

        let mut filename_buf = [0u16; 260];
        let len = GetModuleFileNameExW(
            Some(process_handle),
            Some(mod_handle),
            &mut filename_buf);
        let path = OsString::from_wide(&filename_buf[..len as usize]);
        let file_name = Path::new(&path).file_name().unwrap().to_string_lossy().to_string();

        let module = ProcessModule {
            file_path: path.to_string_lossy().to_string(),
            file_name,
            entry_point: info.EntryPoint as usize as u64,
            size: info.SizeOfImage,
            base: info.lpBaseOfDll as usize as u64,
        };

        modules.push(module);
    }

    Ok(modules)
}