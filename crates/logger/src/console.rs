use core::{fmt::self, mem::zeroed};

use log::{Log, Metadata, Record};
use ntapi::{ntioapi::IO_STATUS_BLOCK, ntpsapi::NtCurrentThreadId};
use spin::Mutex;
use win_platform::{KUserSharedData, syscalls::NtWriteFile, types::PVOID};

use crate::{WinLogger, error::LoggerError};

pub struct ConsoleBuffer(ConsoleBufferInner);

struct ConsoleBufferInner {
    data: [u16; 1024],
    len: usize,
}

impl ConsoleBuffer {
    pub const fn new() -> Self {
        Self(ConsoleBufferInner {
            data: [0u16; 1024],
            len: 0,
        })
    }

    pub const fn as_ptr(&self) -> *const u16 {
        self.0.data.as_ptr()
    }

    pub const fn byte_len(&self) -> u32 {
        (self.0.len * 2) as u32
    }

    pub const fn char_len(&self) -> usize {
        self.0.len
    }

    pub const fn is_empty(&self) -> bool {
        self.0.len == 0
    }

    pub fn reset(&mut self) {
        self.0.len = 0;
    }

    pub fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> fmt::Result {
        self.0.len = 0;
        fmt::Write::write_fmt(self, args)?;
        fmt::Write::write_char(self, '\n')?;
        Ok(())
    }
}

impl fmt::Write for ConsoleBuffer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for ch in s.encode_utf16() {
            if self.0.len >= self.0.data.len() {
                return Err(fmt::Error);
            }
            self.0.data[self.0.len] = ch;
            self.0.len += 1;
        }
        Ok(())
    }
}

pub enum ConsoleMode {
    Existing,
    Attached,
}

pub struct ConsoleLogger(Mutex<ConsoleLoggerInner>);

unsafe impl Send for ConsoleLogger {}
unsafe impl Sync for ConsoleLogger {}

pub struct ConsoleLoggerInner {
    buffer: ConsoleBuffer,
    handle: PVOID,
    mode: ConsoleMode,
}

impl Log for ConsoleLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {

        if !self.enabled(record.metadata()) {
            return;
        }

        let tid = unsafe { NtCurrentThreadId() as u32 };
        let time = KUserSharedData::system_time().to_iso8601();

        let args: fmt::Arguments<'_> = format_args!(
            "{} [tid = 0x{:X}] [{}] {} - {}",
            time, tid, record.level(), record.target(), record.args()
        );

        let _ = self.log_fmt(args);
    }

    fn flush(&self) {}
}

impl ConsoleLoggerInner {
    fn write_existing(&mut self, args: fmt::Arguments<'_>) -> Result<usize, LoggerError> {
        self.buffer.write_fmt(args).map_err(LoggerError::Fmt)?;

        if self.buffer.is_empty() {
            return Ok(0);
        }

        let mut io_block: IO_STATUS_BLOCK = unsafe { zeroed() };

        NtWriteFile(
            self.handle,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &mut io_block,
            self.buffer.as_ptr() as _,
            self.buffer.byte_len(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        )
        .ok()
        .map_err(LoggerError::IoConsole)?;

        Ok((io_block.Information / 2) as usize)
    }

    fn write_attached(&mut self, _args: fmt::Arguments<'_>) -> Result<usize, LoggerError> {
        Ok(0)
    }
}

impl ConsoleLogger {
    pub fn new(handle: PVOID, mode: ConsoleMode) -> Self {
        Self(Mutex::new(ConsoleLoggerInner {
            buffer: ConsoleBuffer::new(),
            handle,
            mode,
        }))
    }
}

impl WinLogger for ConsoleLogger {
    fn log_fmt(&self, args: fmt::Arguments<'_>) -> Result<(), LoggerError> {
        let mut guard = self.0.lock();
        match guard.mode {
            ConsoleMode::Existing => {
                guard.write_existing(args)?;
            }
            ConsoleMode::Attached => {
                guard.write_attached(args)?;
            }
        }
        Ok(())
    }

    fn flush(&self) -> Result<(), LoggerError> {
        Ok(())
    }
}