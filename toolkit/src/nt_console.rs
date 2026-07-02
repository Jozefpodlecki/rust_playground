use core::cell::SyncUnsafeCell;
use core::fmt::{self, Write};
use core::sync::atomic::{AtomicBool, Ordering};
use ntapi::ntioapi::{IO_STATUS_BLOCK, NtWriteFile};
use ntapi::ntpebteb::PEB;
use winapi::shared::minwindef::{BOOL, DWORD};
use winapi::shared::ntdef::{HANDLE, NTSTATUS, PVOID};
use winapi::shared::ntstatus::STATUS_INVALID_HANDLE;
use winapi::um::consoleapi::{GetConsoleMode, GetConsoleOutputCP, WriteConsoleW};
use winapi::um::fileapi::GetFileType;
use crate::get_peb;
use crate::{println, u16_stack_string::U16CStackString};
use winapi::um::winbase::FILE_TYPE_CHAR;

pub const STD_OUTPUT_HANDLE: u32 = 0xFFFFFFF5;
pub const CP_UTF8: u32 = 65001;

pub static mut OUTPUT_HANDLE: SyncUnsafeCell<HANDLE> = SyncUnsafeCell::new(core::ptr::null_mut());

pub struct ConsoleMutex(AtomicBool);

impl ConsoleMutex {
    pub const fn new() -> Self {
        Self(AtomicBool::new(false))
    }

    pub fn lock<'a>(&'a self) -> MutexGuard<'a> {
        while self.0.swap(true, Ordering::Acquire) {
            core::hint::spin_loop();
        }
        MutexGuard(self)
    }

    pub fn unlock(&self) {
        self.0.store(false, Ordering::Release);
    }
}

pub struct MutexGuard<'a>(&'a ConsoleMutex);

impl Drop for MutexGuard<'_> {
    fn drop(&mut self) {
        self.0.unlock();
    }
}

pub static CONSOLE_MUTEX: ConsoleMutex = ConsoleMutex::new();

pub fn get_output_handle() -> HANDLE {
    unsafe {
        let peb_ptr = get_peb();
        let peb = &*peb_ptr;
        let process_params_ptr = peb.ProcessParameters;
        
        if process_params_ptr.is_null() {
            return core::ptr::null_mut();
        }

        let process_params = &*process_params_ptr;
        
        process_params.StandardOutput
    }
}

pub fn write_console_utf8_with_nt_write(
    handle: HANDLE,
    buffer: *const u8,
    chars_to_write: u32,
    chars_written: *mut u32,
) -> NTSTATUS {
    unsafe {
        let mut io_status_block: IO_STATUS_BLOCK = core::mem::zeroed();
        
        if handle.is_null() || buffer.is_null() || chars_to_write == 0 {
            return STATUS_INVALID_HANDLE;
        }
        
        let bytes_to_write = chars_to_write;
        
        let status = NtWriteFile(
            handle,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &mut io_status_block,
            buffer as PVOID,
            bytes_to_write,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        
        if status >= 0 && !chars_written.is_null() {
            let bytes_written = io_status_block.Information as u32;
            *chars_written = bytes_written / 2;
        }
        
        status
    }
}

pub fn write_console_utf16_with_nt_write(
    handle: HANDLE,
    buffer: *const u16,
    chars_to_write: u32,
    chars_written: *mut u32,
) -> NTSTATUS {
    unsafe {
        let mut io_status_block: IO_STATUS_BLOCK = core::mem::zeroed();
        
        if handle.is_null() || buffer.is_null() || chars_to_write == 0 {
            return STATUS_INVALID_HANDLE;
        }
        
        let bytes_to_write = chars_to_write * 2;
        
        let status = NtWriteFile(
            handle,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &mut io_status_block,
            buffer as PVOID,
            bytes_to_write,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        
        if status >= 0 && !chars_written.is_null() {
            let bytes_written = io_status_block.Information as u32;
            *chars_written = bytes_written / 2;
        }
        
        status
    }
}

pub fn get_console_encoding(handle: HANDLE) -> Option<u32> {
    unsafe {
        let file_type = GetFileType(handle);
        if file_type != FILE_TYPE_CHAR {
            return None;
        }

        let mut mode: DWORD = 0;
        let is_console = GetConsoleMode(handle, &mut mode) != 0;

        if is_console {
            // Input or output?
            // GetConsoleMode succeeds for both input and output
            // Check if it's an input console by trying GetNumberOfConsoleInputEvents
            // Or simply assume output for stdout
            Some(GetConsoleOutputCP())
        } else {
            // Not a console handle
            None
        }
    }
}

pub fn get_output_encoding(handle: HANDLE) -> u32 {
    unsafe {
        let file_type = GetFileType(handle);
        if file_type == FILE_TYPE_CHAR {
            let mut mode: DWORD = 0;
            if GetConsoleMode(handle, &mut mode) != 0 {
                return GetConsoleOutputCP();
            }
        }
        // Fallback - default to UTF-8
        CP_UTF8
    }
}

pub fn is_console_utf8(handle: HANDLE) -> bool {
    get_output_encoding(handle) == CP_UTF8
}

pub fn write_console_utf16_with_writeconsolew(
    handle: HANDLE,
    buffer: *const u16,
    chars_to_write: u32,
    chars_written: *mut u32,
) -> BOOL {
    unsafe {
        WriteConsoleW(
            handle,
            buffer as *const winapi::ctypes::c_void,
            chars_to_write,
            chars_written,
            core::ptr::null_mut(),
        )
    }
}

pub struct NtConsole;

impl NtConsole {
    pub fn writeln(text: &str) -> Result<u32, NTSTATUS> {
        let written = Self::write(text)?;
        let newline_written = Self::write("\r\n")?;
        Ok(written + newline_written)
    }

    pub fn write(text: &str) -> Result<u32, NTSTATUS> {
        let _guard = CONSOLE_MUTEX.lock();
        
        let handle = unsafe {
            let handle_ref = OUTPUT_HANDLE.get_mut();
            if handle_ref.is_null() {
                *handle_ref = get_output_handle();
            }
            *handle_ref
        };

        if text.is_empty() {
            return Ok(0);
        }

        let str = match U16CStackString::<260>::from_str(text) {
            Some(s) => s,
            None => return Err(STATUS_INVALID_HANDLE),
        };
        
        let mut written = 0;
        let status = unsafe {
            write_console_utf16_with_nt_write(
                handle,
                str.as_ptr(),
                str.len() as u32,
                &mut written,
            )
        };
        
        if status >= 0 {
            Ok(written)
        } else {
            Err(status)
        }
    }
}

impl Write for NtConsole {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        Self::write(s).map_err(|_| fmt::Error)?;
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut console = $crate::NtConsole;
        let _ = core::fmt::Write::write_fmt(&mut console, core::format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut console = $crate::NtConsole;
        let _ = core::fmt::Write::write_fmt(&mut console, core::format_args!($($arg)*));
        let _ = core::fmt::Write::write_str(&mut console, "\r\n");
    }};
}