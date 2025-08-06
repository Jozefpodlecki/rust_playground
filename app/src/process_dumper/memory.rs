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

    let file_path_str = OsString::from_wide(&filename_buf[..len as usize])
        .to_string_lossy()
        .to_string();

    let file_path = PathBuf::from(&file_path_str);
    let file_name = Path::new(&file_path)
        .file_name()
        .map(|os_str| os_str.to_string_lossy().to_string())
        .unwrap_or_default();

    let is_dll = file_path.extension().filter(|&pr| pr == "dll").is_some();

    let module = ProcessModule {
        order: 0,
        is_dll,
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
    let module_info_size = std::mem::size_of::<MODULEINFO>() as u32;
    
    let mut modules = Vec::with_capacity(count);
    info!("Fetching {} modules", count);

    for (order, &module_handle) in module_handles.into_iter().enumerate() {
        let mut mod_info = MODULEINFO::default();

        GetModuleInformation(
            process_handle,
            module_handle,
            &mut mod_info,
            module_info_size)
            .map_err(|e| anyhow!("GetModuleInformation failed: {:?}", e))?;

        let mut filename_buf = [0u16; 260];
        let len = GetModuleFileNameExW(
            Some(process_handle),
            Some(module_handle),
            &mut filename_buf);
        if len == 0 {
            continue;
        }

        let file_path_str = OsString::from_wide(&filename_buf[..len as usize])
            .to_string_lossy()
            .to_string();

        let file_path = PathBuf::from(&file_path_str);
        let file_name = Path::new(&file_path)
            .file_name()
            .map(|os_str| os_str.to_string_lossy().to_string())
            .unwrap_or_default();

        let is_dll = file_path.extension().filter(|&pr| pr == "dll").is_some();

        let module = ProcessModule {
            order: order as u16,
            is_dll,
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

pub struct MemoryRegionIterator {
    handle: HANDLE,
    address: usize,
    mbi: MEMORY_BASIC_INFORMATION,
    mbi_size: usize,
}

impl MemoryRegionIterator {
    pub fn new(handle: HANDLE) -> Self {
        Self {
            handle,
            address: 0,
            mbi: MEMORY_BASIC_INFORMATION::default(),
            mbi_size: std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
        }
    }
}

impl Iterator for MemoryRegionIterator {
    type Item = Result<(MemoryBlock, Vec<u8>)>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if VirtualQueryEx(
                self.handle,
                Some(self.address as *const _),
                &mut self.mbi,
                self.mbi_size,
            ) == 0
            {
                return None;
            }

            let base = self.mbi.BaseAddress as usize;
            let size = self.mbi.RegionSize;

            self.address = base.saturating_add(size);

            let readable_flags = PAGE_READONLY
                | PAGE_READWRITE
                | PAGE_WRITECOPY
                | PAGE_EXECUTE_READ
                | PAGE_EXECUTE_READWRITE
                | PAGE_EXECUTE_WRITECOPY;

            let is_readable = self.mbi.State == MEM_COMMIT
                && !self.mbi.Protect.contains(PAGE_NOACCESS)
                && !self.mbi.Protect.contains(PAGE_GUARD)
                && (self.mbi.Protect & readable_flags != PAGE_PROTECTION_FLAGS(0));

            let mut buffer = Vec::new();

            if is_readable {
                buffer = Vec::with_capacity(size);
                let mut bytes_read = 0usize;

                let success = ReadProcessMemory(
                    self.handle,
                    self.mbi.BaseAddress,
                    buffer.as_mut_ptr() as *mut c_void,
                    size,
                    Some(&mut bytes_read as *mut usize),
                );

                if success.is_err() {
                    return Some(Err(anyhow!(
                        "ReadProcessMemory failed at 0x{:X}: {:?}",
                        base,
                        success.unwrap_err()
                    )));
                }

                buffer.truncate(bytes_read);
            }

            let protect = self.mbi.Protect;
            let is_readable_flag = protect & readable_flags != PAGE_PROTECTION_FLAGS(0);
            let is_executable_flag = protect & (PAGE_EXECUTE
                | PAGE_EXECUTE_READ
                | PAGE_EXECUTE_READWRITE
                | PAGE_EXECUTE_WRITECOPY) != PAGE_PROTECTION_FLAGS(0);

            let block = MemoryBlock {
                base: base as u64,
                size: size as u64,
                state: self.mbi.State.0,
                protect: self.mbi.Protect.0,
                module: None,
                is_readable: is_readable_flag,
                is_executable: is_executable_flag,
            };

            Some(Ok((block, buffer)))
        }
    }
}