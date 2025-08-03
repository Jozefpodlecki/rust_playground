#![allow(unsafe_op_in_unsafe_fn)]

use std::ffi::{c_void, OsString};
use std::fs::File;
use std::os::windows::ffi::OsStringExt;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;
use anyhow::*;
use log::info;
use ntapi::ntpsapi::{NtResumeProcess, NtSuspendProcess};
use ntapi::ntrtl::RtlNtStatusToDosError;
use windows::core::{w, PCWSTR, PWSTR};
use windows::Win32::System::ProcessStatus::{EnumProcessModulesEx, GetModuleFileNameExW, GetModuleInformation, LIST_MODULES_ALL, MODULEINFO};
use windows::Win32::System::Threading::{CreateProcessW, IsWow64Process, OpenProcess, TerminateProcess, PROCESS_ALL_ACCESS, PROCESS_CREATION_FLAGS, PROCESS_INFORMATION, STARTF_USESHOWWINDOW, STARTUPINFOW};
use windows::Win32::Foundation::{CloseHandle, HANDLE, HMODULE, STATUS_SUCCESS, WIN32_ERROR};
use windows::Win32::System::Diagnostics::Debug::{FormatMessageW, ReadProcessMemory, FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_IGNORE_INSERTS};
use windows::Win32::System::Memory::{VirtualQuery, VirtualQueryEx, MEMORY_BASIC_INFORMATION, MEM_COMMIT, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE, PAGE_EXECUTE_WRITECOPY, PAGE_GUARD, PAGE_NOACCESS, PAGE_READONLY, PAGE_READWRITE, PAGE_WRITECOPY};
use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;
use widestring::U16CString;
use bincode::{Decode, Encode};
use windows::core::PCSTR;
use windows::Win32::System::SystemInformation::{GetVersionExW, OSVERSIONINFOEXW, OSVERSIONINFOW};

use crate::types::RunArgs;

#[derive(Debug, Decode, Encode, Clone)]
pub struct ProcessModule {
    pub file_path: String,
    pub file_name: String,
    pub entry_point: u32,
    pub size: u32,
    pub base: u32,
}

#[derive(Debug, Decode, Encode, Clone)]
pub struct MemoryBlock {
    pub size: u32,
    pub base: u32,
    pub state: u32,
    pub protect: u32,
    pub data: Vec<u8>,
    pub module: Option<ProcessModule>
}

pub unsafe fn get_windows_version() -> Result<String> {
    let mut os_info = OSVERSIONINFOW::default();
    os_info.dwOSVersionInfoSize = std::mem::size_of::<OSVERSIONINFOW>() as u32;

    GetVersionExW(&mut os_info as *mut _ as *mut OSVERSIONINFOW)?;
    let version = format!(
        "{}.{}.{}",
        os_info.dwMajorVersion,
        os_info.dwMinorVersion,
        os_info.dwBuildNumber
    );

    Ok(version)
}

fn match_module<'a>(
    base: u32,
    modules: &'a [ProcessModule],
) -> Option<&'a ProcessModule> {
    modules.iter().find(|module| {
        let module_start = base as usize;
        let module_end = module_start + module.size as usize;
        let block_base = base as usize;

        block_base >= module_start && block_base < module_end
    })
}

pub unsafe fn monitor_address(h_process: HANDLE, addr: usize) -> Result<()> {
    let mut orig = [0u8; 4];
    let mut bytes_read = 0usize;

    ReadProcessMemory(
        h_process,
        addr as *const _,
        orig.as_mut_ptr() as _,
        orig.len(),
        Some(&mut bytes_read),
    ).ok();

    info!("Bytes {bytes_read}");

    loop {
        let mut new = [0u8; 4];
        ReadProcessMemory(
            h_process,
            addr as *const _,
            new.as_mut_ptr() as _,
            new.len(),
            Some(&mut bytes_read),
        ).ok();
        info!("{orig:?} -> {new:?}");

        if orig != new {
            info!("{orig:?} -> {new:?}");
            break;
        }

        sleep(Duration::from_millis(1000));
    }

    sleep(Duration::from_millis(1500));

    Ok(())
}

pub unsafe fn dump_process(args: RunArgs) -> Result<()> {

    let RunArgs {
        exe_path,
        exe_args,
        ..
    } = args;

    let startup_info = STARTUPINFOW {
        dwFlags: STARTF_USESHOWWINDOW,
        wShowWindow: SW_HIDE.0 as u16,
        ..Default::default()
    };
    let mut process_information = PROCESS_INFORMATION::default();
    let mut module_handles: [HMODULE; 64] = [HMODULE::default(); 64];

    let command_line = exe_args.join(" ");
    let wide_cmd = U16CString::from_str(&command_line)?;
    let command_line = PWSTR(wide_cmd.as_ptr() as *mut u16);

    let process_id = CreateProcessW(
        PCWSTR::null(),
         Some(command_line),
        None,
        None,
        false,
        PROCESS_CREATION_FLAGS::default(),
        None,
        PCWSTR::null(),
        &startup_info,
        &mut process_information,
    ).map(|_| process_information.dwProcessId)?;

    let process_handle: HANDLE = OpenProcess(
        PROCESS_ALL_ACCESS,
        false,
        process_id)?;
    let process_handle_c_void = process_handle.0 as *mut ntapi::winapi::ctypes::c_void;

    let mut is_64 = false.into();
    IsWow64Process(process_handle, &mut is_64)?;
    info!("is_64: {is_64:?}");

    info!("Waiting for process to start");
    sleep(Duration::from_secs(1));
    let mut address = 0 as usize;
    let mut cb_needed = 0u32;
    
    EnumProcessModulesEx(
        process_handle,
        module_handles.as_mut_ptr(),
        std::mem::size_of_val(&module_handles) as u32,
        &mut cb_needed,
        LIST_MODULES_ALL,
    )?;

    let actual_length = (cb_needed as usize) / std::mem::size_of::<HMODULE>();
    let module_handles = module_handles[..actual_length].to_vec();

    let module_info_size = std::mem::size_of::<MODULEINFO>() as u32;
    let mut process_modules = vec![];

    for module_handle in module_handles {
        let mut mod_info = MODULEINFO::default();
        GetModuleInformation(process_handle, module_handle, &mut mod_info, module_info_size)?;
        
        let mut file_name_buf = [0u16; 260];
        GetModuleFileNameExW(Some(process_handle), Some(module_handle), &mut file_name_buf);
        let file_path = OsString::from_wide(&file_name_buf[..file_name_buf.len() as usize]).to_string_lossy().to_string();

        let file_name = Path::new(&file_path).file_name().unwrap().to_string_lossy().to_string();

        let module = ProcessModule {
            file_path,
            file_name,
            entry_point: mod_info.EntryPoint as usize as u32,
            size: mod_info.SizeOfImage,
            base: mod_info.lpBaseOfDll as usize as u32,
        };

        info!("Module: {} base: 0x{:X} size: 0x{:X}", module.file_name, module.base, module.size);
        
        process_modules.push(module);
    }

    let main_module = process_modules.first().unwrap();
    let address_to_watch = main_module.base + 0x45f000;
    info!("0x{address_to_watch:X}");
    // debug_memory_region(process_handle, address_to_watch as usize)?;
    // monitor_address(process_handle, address_to_watch as usize)?;
    
    let status = NtSuspendProcess(process_handle_c_void);

    if status != STATUS_SUCCESS.0 {
        let win32_err = unsafe { RtlNtStatusToDosError(status) };

        let mut buf = [0u16; 512];
        let pwstr_buf = PWSTR(buf.as_mut_ptr());
        let len = FormatMessageW(
            FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
            None,
            WIN32_ERROR(win32_err).0,
            0,
            pwstr_buf,
            buf.len() as u32,
            None,
        );
    }

    let mbi_size = std::mem::size_of::<MEMORY_BASIC_INFORMATION>();
    let mut blocks = vec![];
    
    loop {
        let mut mbi = MEMORY_BASIC_INFORMATION::default();

        if VirtualQueryEx(
            process_handle,
            Some(address as *const _),
            &mut mbi,
            mbi_size,
        ) == 0 {
            break;
        }

        address = (mbi.BaseAddress as usize).saturating_add(mbi.RegionSize);

        let is_readable = mbi.State == MEM_COMMIT
            && mbi.Protect.contains(PAGE_NOACCESS)
            && mbi.Protect.contains(PAGE_GUARD)
            && (mbi.Protect.contains(PAGE_READONLY | PAGE_READWRITE | PAGE_WRITECOPY | PAGE_EXECUTE_READ | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY));

        info!("Reading block: address: {:?} {} size: {}", mbi.BaseAddress, address, mbi.RegionSize);
        let mut buffer = vec![];

        if is_readable {
            buffer = vec![0u8; mbi.RegionSize];
            let mut bytes_read = 0usize;

            ReadProcessMemory(
                process_handle,
                mbi.BaseAddress,
                buffer.as_mut_ptr() as _,
                mbi.RegionSize,
                Some(&mut bytes_read as *mut usize),
            )?;
        }

        let base = mbi.BaseAddress as usize as u32;
        let block = MemoryBlock {
            base: mbi.BaseAddress as usize as u32,
            size: mbi.RegionSize as usize as u32,
            state: mbi.State.0,
            protect: mbi.Protect.0,
            data: buffer,
            module: match_module(base, &process_modules).cloned()
        };

        blocks.push(block);
    }

    NtResumeProcess(process_handle_c_void);
    TerminateProcess(process_handle, 0)?;
    CloseHandle(process_handle)?;

    let file_name = Path::new(&exe_path).file_stem().unwrap().to_string_lossy();
    let output_bin_path = format!("{file_name}.bin");
    let config = bincode::config::standard();
    let mut writer = File::create(&output_bin_path)?;
    bincode::encode_into_std_write(blocks, &mut writer, config)?;
// 
    Ok(())

}

pub unsafe fn debug_memory_region(h_process: HANDLE, addr: usize) -> Result<()> {
    let mut mbi = MEMORY_BASIC_INFORMATION::default();
    let size = std::mem::size_of::<MEMORY_BASIC_INFORMATION>();

    let result = VirtualQueryEx(h_process, Some(addr as *const _), &mut mbi, size);
    if result == 0 {
        bail!("VirtualQueryEx failed at address {:?}", addr);
    }

    println!("Memory Region @ {:p}", mbi.BaseAddress);
    println!("  Region Size: 0x{:X} bytes", mbi.RegionSize);
    print!("  State: ");
    match mbi.State {
        MEM_COMMIT => println!("MEM_COMMIT"),
        MEM_FREE => println!("MEM_FREE"),
        MEM_RESERVE => println!("MEM_RESERVE"),
        _ => println!("Unknown ({:?})", mbi.State),
    }
    print!("  Protection: ");
    println!();

    Ok(())
}