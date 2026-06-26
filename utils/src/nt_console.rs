use core::cell::SyncUnsafeCell;
use core::fmt::{self, Write};
use core::sync::atomic::{AtomicBool, Ordering};
use ntapi::ntioapi::{IO_STATUS_BLOCK, NtWriteFile};
use ntapi::ntpebteb::PEB;
use winapi::shared::ntdef::{HANDLE, NTSTATUS, PVOID};
use winapi::shared::ntstatus::STATUS_INVALID_HANDLE;
use crate::{println, u16_stack_string::U16CStackString};

pub const STD_OUTPUT_HANDLE: u32 = 0xFFFFFFF5;

pub static mut OUTPUT_HANDLE: SyncUnsafeCell<HANDLE> = SyncUnsafeCell::new(core::ptr::null_mut());

pub struct Mutex {
    locked: AtomicBool,
}

impl Mutex {
    pub const fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
        }
    }

    pub fn lock(&self) -> MutexGuard {
        while self.locked.swap(true, Ordering::Acquire) {
            core::hint::spin_loop();
        }
        MutexGuard { mutex: self }
    }

    pub fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}

pub struct MutexGuard<'a> {
    mutex: &'a Mutex,
}

impl Drop for MutexGuard<'_> {
    fn drop(&mut self) {
        self.mutex.unlock();
    }
}

pub static CONSOLE_MUTEX: Mutex = Mutex::new();

#[unsafe(naked)]
pub unsafe fn get_peb() -> *mut PEB {
    core::arch::naked_asm!(
        "mov rax, gs:[0x60]",
        "ret"
    );
}

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
macro_rules! println {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut console = $crate::NtConsole;
        let _ = core::fmt::Write::write_fmt(&mut console, core::format_args!($($arg)*));
        let _ = core::fmt::Write::write_str(&mut console, "\r\n");
    }};
}