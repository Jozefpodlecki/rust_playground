
use alloc::{boxed::Box, vec::Vec};
use log::{Log, Metadata, Record};
use ntapi::ntpsapi::NtCurrentThreadId;
use win_platform::KUserSharedData;
use core::fmt;

use crate::{WinLogger, error::LoggerError};

pub struct AggregatedLogger(pub Box<[Box<dyn WinLogger>]>);

impl Log for AggregatedLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {

        if !self.enabled(record.metadata()) {
            return;
        }

        let _ = self.log_inner(record);
    }

    fn flush(&self) {
        let _ = self.flush();
    }
}

impl AggregatedLogger {
    pub fn new(loggers: Vec<Box<dyn WinLogger>>) -> Self {
        Self(loggers.into_boxed_slice())
    }

    fn log_inner(&self, record: &Record) -> Result<(), LoggerError> {

        let tid = unsafe { NtCurrentThreadId() as u32 };
        let time = KUserSharedData::system_time().to_iso8601();

        let args: fmt::Arguments<'_> = format_args!(
            "{} [tid = 0x{:X}] [{}] {} - {}",
            time, tid, record.level(), record.target(), record.args()
        );

        for logger in &self.0 {
            let _ = logger.log_fmt(args);
        }

        Ok(())
    }

    fn flush(&self) -> Result<(), LoggerError> {
        
        for logger in &self.0 {
            let _ = WinLogger::flush(&**logger);
        }

        Ok(())
    }
}

impl Drop for AggregatedLogger {
    fn drop(&mut self) {
        let loggers = core::mem::take(&mut self.0);
        for logger in loggers.iter() {
            let _ = WinLogger::flush(&**logger);
        }
        drop(loggers);
    }
}