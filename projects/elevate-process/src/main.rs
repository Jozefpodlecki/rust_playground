#![no_std]
#![no_main]
#![windows_subsystem = "console"]
#![allow(unused)]

use core::{panic::PanicInfo, ptr::null_mut};

use ntapi::{ntobapi::NtClose, ntpsapi::{NtCurrentProcess, NtTerminateProcess}, ntseapi::{NtOpenProcessToken, NtQueryInformationToken}};
use toolkit::*;
use winapi::{shared::{minwindef::{DWORD, FALSE, HINSTANCE, HKEY, MAX_PATH, TRUE}, ntdef::{HANDLE, NTSTATUS}, ntstatus::STATUS_SUCCESS, windef::HWND}, um::{handleapi::CloseHandle, libloaderapi::GetModuleFileNameW, processthreadsapi::{ExitProcess, GetCurrentProcess, OpenProcessToken}, securitybaseapi::GetTokenInformation, shellapi::{SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW}, winnt::{TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation}, winuser::SW_SHOWNORMAL}};

extern crate builtins;

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

pub fn elevate_with_runas(exe_path: *const u16) -> bool {
    let verb = U16CStackString::<10>::from_str("runas").unwrap();

    let mut sei = SHELLEXECUTEINFOW {
        cbSize: core::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS,
        hwnd: null_mut() as HWND,
        lpVerb: verb.as_ptr(),
        lpFile: exe_path,
        lpParameters: null_mut(),
        lpDirectory: null_mut(),
        nShow: SW_SHOWNORMAL,
        hInstApp: null_mut() as HINSTANCE,
        lpIDList: null_mut(),
        lpClass: null_mut(),
        hkeyClass: null_mut() as HKEY,
        dwHotKey: 0,
        hMonitor: null_mut() as HANDLE,
        hProcess: null_mut() as HANDLE,
    };

    let result = unsafe { ShellExecuteExW(&mut sei) };
    
    if result == TRUE && !sei.hProcess.is_null() {
        unsafe { CloseHandle(sei.hProcess as HANDLE); }
        true
    } else {
        false
    }
}

#[derive(Debug)]
pub enum TokenError {
    OpenFailed(NTSTATUS),
    QueryFailed(NTSTATUS),
    CloseFailed(NTSTATUS),
    InvalidToken,
}


pub fn is_process_elevated() -> Result<bool, TokenError> {
    let mut token: HANDLE = null_mut();
    let status = unsafe { NtOpenProcessToken(NtCurrentProcess, TOKEN_QUERY, &mut token) };
    
    if status != STATUS_SUCCESS {
        return Err(TokenError::OpenFailed(status));
    }

    let mut elevation: TOKEN_ELEVATION = unsafe { core::mem::zeroed() };
    let mut return_length: DWORD = 0;
    let result = unsafe {
        NtQueryInformationToken(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            core::mem::size_of::<TOKEN_ELEVATION>() as DWORD,
            &mut return_length,
        )
    };

    let close_status = unsafe { NtClose(token) };
    if close_status != STATUS_SUCCESS {
        return Err(TokenError::CloseFailed(close_status));
    }

    if status != STATUS_SUCCESS {
        return Err(TokenError::QueryFailed(status));
    }

    Ok(elevation.TokenIsElevated != 0)
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {

    let handle = get_output_handle();
    let encoding = get_console_encoding(handle);
    println!("{encoding:?}");

    let elevated = match is_process_elevated() {
        Ok(value) => value,
        Err(err) => return 1,
    };
    
    {
        let test = U16CStackString::<10>::from_str("test\n").unwrap();
        write_console_utf16_with_writeconsolew(
            handle,
            test.as_ptr(),
            test.len() as u32 * 2,
            null_mut());
    }
    
    if elevated {
        Sleeper::sleep(1000);
        println!("elevated");
        Sleeper::sleep(5000);
        return 0
    }
    else {
        println!("not elevated");
    }

    let peb = ProcessEnvironmentBlock::current_process();
    let executable_path = peb.executable_path();
    if elevate_with_runas(executable_path.as_ptr()) {
        return 0;
    }
    
    0
}