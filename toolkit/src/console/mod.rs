use core::fmt::{self, Write};

use winapi::shared::ntdef::{HANDLE, NTSTATUS, PVOID};

use crate::{Mutex, console::strategy::{ConsoleWriter, DeviceIoControlStrategy, NtWriteStrategy}};

mod strategy;

pub type DefaultConsole = ConsoleWriter<NtWriteStrategy>;
// pub type DefaultConsole = ConsoleWriter<DeviceIoControlStrategy>;
pub static CONSOLE: DefaultConsole = ConsoleWriter::new();

pub use strategy::get_output_handle;

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
        CONSOLE.writeln(text)
    }

    pub fn write(text: &str) -> Result<u32, NTSTATUS> {
        CONSOLE.write(text)
    }
}

impl Write for NtConsole {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        Self::write(s).map_err(|_| fmt::Error)?;
        Ok(())
    }
}

pub static LOCK: Mutex<()> = Mutex::new(());

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut console = $crate::NtConsole;
        let _ = $crate::LOCK.lock();
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
        let _ = $crate::LOCK.lock();
        let _ = core::fmt::Write::write_fmt(&mut console, core::format_args!($($arg)*));
        let _ = core::fmt::Write::write_str(&mut console, "\r\n");
    }};
}