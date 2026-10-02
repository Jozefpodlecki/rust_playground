use core::fmt;
use core::io::Write;

use crate::{WinLogger, error::LoggerError};

use log::Log;
use ntapi::ntpsapi::NtCurrentThreadId;
use spin::Mutex;
use win_platform::{KUserSharedData, fs::File, types::UnicodeString, utils::BufWriter};

pub struct FileLogger<const N: usize>(Mutex<BufWriter<File, N>>);

impl<const N: usize> FileLogger<N> {
    pub fn new(path: UnicodeString) -> Result<Self, LoggerError> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        Ok(Self(Mutex::new(writer)))
    }
}

impl<const N: usize> Log for FileLogger<N> {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
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

    fn flush(&self) {
        let _ = self.0.lock().flush();
    }
}

impl<const N: usize> WinLogger for FileLogger<N> {
    fn log_fmt(&self, args: fmt::Arguments<'_>) -> Result<(), LoggerError> {
        let mut guard = self.0.lock();
        core::io::Write::write_fmt(&mut *guard, args)?;
        core::io::Write::write_all(&mut *guard, b"\n")?;
        Ok(())
    }

    fn flush(&self) -> Result<(), LoggerError> {
        self.0.lock().flush()?;
        Ok(())
    }
}