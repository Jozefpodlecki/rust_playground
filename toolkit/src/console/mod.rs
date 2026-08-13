use core::{fmt::{self, Write}, mem::zeroed, ptr::null_mut, sync::atomic::{AtomicPtr, Ordering}};

use ntapi::{ntexapi::NtReleaseMutant, ntobapi::NtWaitForSingleObject};
use widestring::{U16CStr, u16cstr};
use winapi::{shared::ntdef::{HANDLE, NTSTATUS, OBJ_CASE_INSENSITIVE, OBJECT_ATTRIBUTES, PVOID, UNICODE_STRING}, um::winnt::MUTANT_ALL_ACCESS};

use crate::{Mutex, console::{strategy::{DeviceIoControlStrategy, NtWriteStrategy}, writer::ConsoleWriter}};

mod strategy;
mod writer;

pub type DefaultConsole = ConsoleWriter<NtWriteStrategy>;
// pub type DefaultConsole = ConsoleWriter<DeviceIoControlStrategy>;
pub static CONSOLE: DefaultConsole = ConsoleWriter::new();

pub use writer::get_output_handle;

pub trait WriteStrategy {
    fn write(
        handle: HANDLE,
        buffer: PVOID,
        chars_to_write: u32,
        chars_written: *mut u32,
    ) -> NTSTATUS;
}

pub struct NtConsole;

impl NtConsole {
    pub fn writeln(text: &str) -> Result<u32, NTSTATUS> {
        let result = CONSOLE.writeln(text);
        CONSOLE.clear();
        result
    }

    pub fn write(text: &str) -> Result<u32, NTSTATUS> {
        let result = CONSOLE.write(text);
        CONSOLE.clear();
        result
    }
}

impl Write for NtConsole {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        Self::write(s).map_err(|_| fmt::Error)?;
        Ok(())
    }
}

// pub static LOCK: Mutex<()> = Mutex::new(());

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut console = $crate::NtConsole;
        // let _ = $crate::LOCK.lock();
        let _ = core::fmt::Write::write_fmt(&mut console, core::format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\r\n")
    };
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut console = $crate::NtConsole;
        // let _ = $crate::LOCK.lock();
        let _ = core::fmt::Write::write_fmt(&mut console, core::format_args!($($arg)*));
        let _ = core::fmt::Write::write_str(&mut console, "\r\n");
    }};
}

static LOCK: AtomicPtr<winapi::ctypes::c_void> = AtomicPtr::new(null_mut());
static mut NAME: &U16CStr = u16cstr!("\\BaseNamedObjects\\CONSOLE");
static mut NAME_UC: UNICODE_STRING = UNICODE_STRING {
    Length: 56,
    MaximumLength: 58,
    Buffer: unsafe { NAME.as_ptr() as _ },
};
static mut CONSOLE_LOCK_OBJ_ATTR: OBJECT_ATTRIBUTES = OBJECT_ATTRIBUTES {
    Length: core::mem::size_of::<OBJECT_ATTRIBUTES>() as _,
    RootDirectory: null_mut(),
    ObjectName: unsafe { &mut NAME_UC },
    Attributes: OBJ_CASE_INSENSITIVE,
    SecurityDescriptor: null_mut(),
    SecurityQualityOfService: null_mut(),
};

#[macro_export]
macro_rules! thread_safe_println {
    ($($arg:tt)*) => {{
        let handle = LOCK.load(Ordering::Acquire);
        if handle.is_null() {
            let mut new_handle: HANDLE = null_mut();
            let status = unsafe {
                ntapi::ntexapi::NtCreateMutant(
                    &mut new_handle,
                    winapi::um::winnt::MUTANT_ALL_ACCESS,
                    &mut CONSOLE_LOCK_OBJ_ATTR,
                    0,
                )
            };
            
            if status >= 0 {
                if LOCK.compare_exchange(null_mut(), new_handle, Ordering::Release, Ordering::Acquire).is_ok() {
                } else {
                    unsafe { winapi::um::handleapi::CloseHandle(new_handle); }
                }
            }
        }

        let lock_handle = LOCK.load(Ordering::Acquire);
        unsafe {
            NtWaitForSingleObject(lock_handle, 0, null_mut());
            let _ = core::fmt::Write::write_fmt(&mut console, core::format_args!($($arg)*));
            let _ = core::fmt::Write::write_str(&mut console, "\r\n");
            NtReleaseMutant(lock_handle, null_mut());
        }
    }};
}