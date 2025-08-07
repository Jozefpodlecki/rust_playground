use anyhow::*;
use log::*;
use ntapi::ntpsapi::{NtResumeProcess, NtSuspendProcess};
use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use windows::Win32::System::Threading::{CreateProcessW, IsWow64Process, OpenProcess, TerminateProcess, PROCESS_ALL_ACCESS, PROCESS_INFORMATION, STARTF_USESHOWWINDOW, STARTUPINFOW};
use widestring::U16CString;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOW;
use std::thread::sleep;
use std::time::Duration;

pub unsafe fn spawn_process(exe_args: &[String]) -> Result<(u32, HANDLE)> {
    let startup_info = STARTUPINFOW {
        dwFlags: STARTF_USESHOWWINDOW,
        wShowWindow: SW_SHOW.0 as u16,
        ..Default::default()
    };

    let mut process_info = PROCESS_INFORMATION::default();
    let command_line = exe_args.join(" ");
    let wide_cmd = U16CString::from_str(&command_line)?;
    let command_line = PWSTR(wide_cmd.as_ptr() as *mut u16);

    CreateProcessW(
        None,
        Some(command_line),
        None,
        None,
        false,
        Default::default(),
        None,
        None,
        &startup_info,
        &mut process_info,
    )?;
    
    let handle = OpenProcess(PROCESS_ALL_ACCESS, false, process_info.dwProcessId)?;
    Ok((process_info.dwProcessId, handle))
}

/// Suspends the entire process.
pub unsafe fn suspend_process(process_handle: HANDLE) -> Result<()> {
    let status = NtSuspendProcess(process_handle.0 as *mut _);
    if status == 0 {
        Ok(())
    } else {
        Err(anyhow!("Failed to suspend process, NTSTATUS: {:#X}", status))
    }

    //     if status != STATUS_SUCCESS.0 {
//         let win32_err = unsafe { RtlNtStatusToDosError(status) };

//         let mut buf = [0u16; 512];
//         let pwstr_buf = PWSTR(buf.as_mut_ptr());
//         let len = FormatMessageW(
//             FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
//             None,
//             WIN32_ERROR(win32_err).0,
//             0,
//             pwstr_buf,
//             buf.len() as u32,
//             None,
//         );
//     }
}

/// Resumes a suspended process.
pub unsafe fn resume_process(process_handle: HANDLE) -> Result<()> {
    let status = NtResumeProcess(process_handle.0 as *mut _);
    if status == 0 {
        Ok(())
    } else {
        Err(anyhow!("Failed to resume process, NTSTATUS: {:#X}", status))
    }
}

/// Terminates a process with exit code 0.
pub unsafe fn terminate_process(process_handle: HANDLE) -> Result<()> {
    TerminateProcess(process_handle, 0)?;
    Ok(())
}

pub unsafe fn close_handle(handle: HANDLE) -> Result<()> {
    CloseHandle(handle)?;
    Ok(())
}

/// Checks if the target process is running under WOW64 (i.e., 32-bit process on 64-bit Windows).
pub unsafe fn is_wow64_process(process_handle: HANDLE) -> bool {
    let mut is_wow64 = false.into();
    IsWow64Process(process_handle, &mut is_wow64)
        .ok()
        .map(|_| is_wow64.as_bool())
        .unwrap_or_default()
}

/// Monitors an address in the process memory until the 4 bytes at that address change.
pub unsafe fn monitor_address(process_handle: HANDLE, addr: u64, wait_interval: Duration) -> Result<()> {
    let mut orig = [0u8; 4];
    let mut bytes_read = 0usize;

    ReadProcessMemory(
        process_handle,
        addr as *const _,
        orig.as_mut_ptr() as _,
        orig.len(),
        Some(&mut bytes_read),
    )
    .map_err(|e| anyhow!("ReadProcessMemory failed: {:?}", e))?;

    debug!("Initial bytes at 0x{:X}: {:?}", addr, orig);

    loop {
        let mut new = [0u8; 4];
        ReadProcessMemory(
            process_handle,
            addr as *const _,
            new.as_mut_ptr() as _,
            new.len(),
            Some(&mut bytes_read),
        )
        .map_err(|e| anyhow!("ReadProcessMemory failed: {:?}", e))?;

        if orig != new {
            debug!("Bytes changed at 0x{:X}: {:?} -> {:?}", addr, orig, new);
            break;
        }

        sleep(wait_interval);
    }

    Ok(())
}