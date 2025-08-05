use std::{ffi::c_void, os::windows::ffi::OsStringExt, path::Path, time::Duration};
use anyhow::*;
use log::*;
use windows::Win32::Foundation::{HANDLE, HMODULE};
use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory};
use windows::Win32::System::Memory::{
    VirtualQueryEx, MEMORY_BASIC_INFORMATION, MEM_COMMIT, PAGE_EXECUTE, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE, PAGE_EXECUTE_WRITECOPY, PAGE_GUARD, PAGE_NOACCESS, PAGE_PROTECTION_FLAGS, PAGE_READONLY, PAGE_READWRITE, PAGE_WRITECOPY
};
use windows::Win32::System::ProcessStatus::{EnumProcessModulesEx, GetModuleFileNameExW, GetModuleInformation, LIST_MODULES_ALL, MODULEINFO};
use std::ffi::OsString;
use std::path::PathBuf;

use crate::process_dumper::types::{MemoryBlock, ProcessModule};
use crate::process_dumper::utils::match_module;

pub unsafe fn get_main_module(process_handle: HANDLE) -> Result<Option<ProcessModule>> {
    let mut module_handles: [HMODULE; 1] = [HMODULE::default()];
    let mut cb_needed = 0u32;

    EnumProcessModulesEx(
        process_handle,
        module_handles.as_mut_ptr(),
        std::mem::size_of_val(&module_handles) as u32,
        &mut cb_needed,
        LIST_MODULES_ALL,
    )?;

    if cb_needed == 0 {
        return Ok(None);
    }

    let module_handle = module_handles[0];

    let mut mod_info = MODULEINFO::default();
    let size = std::mem::size_of::<MODULEINFO>() as u32;

    GetModuleInformation(process_handle, module_handle, &mut mod_info, size)
        .map_err(|e| anyhow!("GetModuleInformation failed: {:?}", e))?;

    let mut filename_buf = [0u16; 260];
    let len = GetModuleFileNameExW(Some(process_handle), Some(module_handle), &mut filename_buf);
    if len == 0 {
        return Ok(None);
    }

    let file_path = OsString::from_wide(&filename_buf[..len as usize])
        .to_string_lossy()
        .to_string();

    let file_name = Path::new(&file_path)
        .file_name()
        .map(|os_str| os_str.to_string_lossy().to_string())
        .unwrap_or_default();

    let module = ProcessModule {
        file_path,
        file_name,
        entry_point: mod_info.EntryPoint as usize as u64,
        size: mod_info.SizeOfImage,
        base: mod_info.lpBaseOfDll as usize as u64,
    };

    Ok(Some(module))
}


/// Enumerate modules in the target process and collect information about them.
pub unsafe fn enumerate_modules(process_handle: HANDLE) -> Result<Vec<ProcessModule>> {
    let mut module_handles: [HMODULE; 256] = [HMODULE::default(); 256];
    let mut cb_needed = 0u32;

    EnumProcessModulesEx(
        process_handle,
        module_handles.as_mut_ptr(),
        std::mem::size_of_val(&module_handles) as u32,
        &mut cb_needed,
        LIST_MODULES_ALL,
    )
    .map_err(|e| anyhow!("EnumProcessModulesEx failed: {:?}", e))?;

    let count = (cb_needed as usize) / std::mem::size_of::<HMODULE>();
    let module_handles = &module_handles[..count];

    let mut modules = Vec::with_capacity(count);
    info!("Fetching {} modules", count);

    for &module_handle in module_handles {
        let mut mod_info = MODULEINFO::default();
        let size = std::mem::size_of::<MODULEINFO>() as u32;

        GetModuleInformation(process_handle, module_handle, &mut mod_info, size)
            .map_err(|e| anyhow!("GetModuleInformation failed: {:?}", e))?;

        let mut filename_buf = [0u16; 260];
        let len = GetModuleFileNameExW(Some(process_handle), Some(module_handle), &mut filename_buf);
        if len == 0 {
            continue;
        }

        let file_path = OsString::from_wide(&filename_buf[..len as usize])
            .to_string_lossy()
            .to_string();

        let file_name = Path::new(&file_path)
            .file_name()
            .map(|os_str| os_str.to_string_lossy().to_string())
            .unwrap_or_default();

        let module = ProcessModule {
            file_path,
            file_name,
            entry_point: mod_info.EntryPoint as usize as u64,
            size: mod_info.SizeOfImage,
            base: mod_info.lpBaseOfDll as usize as u64,
        };

        debug!(
            "Module: {} base: 0x{:X} end: 0x{:X} entry_point: 0x{:X}",
            module.file_name,
            module.base,
            module.base + module.size as u64,
            module.entry_point
        );

        modules.push(module);
    }

    Ok(modules)
}

/// Dumps memory regions of the process by querying virtual memory regions,
/// reading committed, readable pages, and collecting them into blocks.
pub unsafe fn dump_memory_regions(process_handle: HANDLE) -> Result<Vec<MemoryBlock>> {
    let mut blocks = Vec::new();
    let mut address = 0usize;
    let mbi_size = std::mem::size_of::<MEMORY_BASIC_INFORMATION>();

    loop {
        let mut mbi = MEMORY_BASIC_INFORMATION::default();

        if VirtualQueryEx(
            process_handle,
            Some(address as *const _),
            &mut mbi,
            mbi_size,
        ) == 0
        {
            break;
        }

        address = (mbi.BaseAddress as usize).saturating_add(mbi.RegionSize);

        debug!(
            "Queried region: base=0x{:X}, size=0x{:X}, state=0x{:X}, protect=0x{:X}",
            mbi.BaseAddress as usize,
            mbi.RegionSize,
            mbi.State.0,
            mbi.Protect.0
        );

        let readable_flags = PAGE_READONLY
            | PAGE_READWRITE
            | PAGE_WRITECOPY
            | PAGE_EXECUTE_READ
            | PAGE_EXECUTE_READWRITE
            | PAGE_EXECUTE_WRITECOPY;

        let is_readable = mbi.State == MEM_COMMIT
            && !mbi.Protect.contains(PAGE_NOACCESS)
            && !mbi.Protect.contains(PAGE_GUARD)
            && (mbi.Protect & readable_flags != PAGE_PROTECTION_FLAGS(0));

        let mut buffer = Vec::new();

        if is_readable {
            debug!("Reading memory at 0x{:X}", mbi.BaseAddress as usize);
            buffer.resize(mbi.RegionSize, 0);
            let mut bytes_read = 0usize;

            ReadProcessMemory(
                process_handle,
                mbi.BaseAddress,
                buffer.as_mut_ptr() as *mut c_void,
                mbi.RegionSize,
                Some(&mut bytes_read as *mut usize),
            )
            .map_err(|e| anyhow!("ReadProcessMemory failed at 0x{:X}: {:?}", mbi.BaseAddress as usize, e))?;

            buffer.truncate(bytes_read);
        }

        let protect = mbi.Protect;
        let is_readable_flag = protect & (PAGE_READONLY
            | PAGE_READWRITE
            | PAGE_WRITECOPY
            | PAGE_EXECUTE_READ
            | PAGE_EXECUTE_READWRITE
            | PAGE_EXECUTE_WRITECOPY) != PAGE_PROTECTION_FLAGS(0);

        let is_executable_flag = protect & (PAGE_EXECUTE
            | PAGE_EXECUTE_READ
            | PAGE_EXECUTE_READWRITE
            | PAGE_EXECUTE_WRITECOPY) != PAGE_PROTECTION_FLAGS(0);

        let base = mbi.BaseAddress as usize as u64;

        let block = MemoryBlock {
            base,
            size: mbi.RegionSize as u64,
            state: mbi.State.0,
            protect: mbi.Protect.0,
            // data: buffer,
            module: None,
            is_readable: is_readable_flag,
            is_executable: is_executable_flag,
        };

        blocks.push(block);
    }

    Ok(blocks)
}

// pub fn search_value_in_memory(blocks: &[&[u8]], target: u32) -> Vec<u64> {
//     let mut matches = Vec::new();
//     let target_bytes = target.to_le_bytes();
//     info!("Searching {:?}", target_bytes);

//     for block in blocks {

//         if block.len() < 4 {
//             continue;
//         }

//         for i in 0..=(block.len() - 4) {
//             if block[i..i + 4] == target_bytes {
//                 let addr = block.base + i as u64;
//                 matches.push(addr);
//             }
//         }
//     }

//     matches
// }