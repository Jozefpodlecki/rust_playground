use core::{cell::SyncUnsafeCell, ops::Deref};

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, vec::Vec};
use sized_dst::DstA64;
#[cfg(feature = "alloc")]
use win_platform::{ProcessEnvironmentBlock, types::{PVOID, UnicodeString}};

use crate::{WinLogger, agg::AggregatedLogger, console::{ConsoleLogger, ConsoleMode}, error::LoggerError, file::FileLogger};

#[cfg(feature = "alloc")]
pub struct LoggerBuilder {
    loggers: Vec<Box<dyn WinLogger>>,
    max_level: log::LevelFilter,
}

#[cfg(feature = "alloc")]
impl LoggerBuilder {
    pub fn new() -> Self {
        Self {
            loggers: Vec::new(),
            max_level: log::LevelFilter::Info,
        }
    }

    pub fn console_current(mut self) -> Result<Self, LoggerError> {
        let peb = ProcessEnvironmentBlock::current_process();
        let handle = peb.standard_output().ok_or_else(|| LoggerError::NotInitialized)?;
        self.loggers.push(Box::new(ConsoleLogger::new(handle, ConsoleMode::Existing)));
        Ok(self)
    }

    pub fn console(mut self, handle: PVOID, mode: ConsoleMode) -> Self {
        self.loggers.push(Box::new(ConsoleLogger::new(handle, mode)));
        self
    }

    pub fn file<const N: usize>(
        mut self,
        path: UnicodeString,
    ) -> Result<Self, LoggerError> {
        let logger: Box<dyn WinLogger> = Box::new(FileLogger::<N>::new(path)?);
        self.loggers.push(logger);
        Ok(self)
    }

    pub fn max_level(mut self, level: log::LevelFilter) -> Self {
        self.max_level = level;
        self
    }

    pub fn build(self) -> Result<&'static AggregatedLogger, LoggerError> {
        let logger = Box::leak(Box::new(AggregatedLogger(self.loggers.into())));
        log::set_logger(logger)?;
        log::set_max_level(self.max_level);
        
        Ok(logger)
    }
}

pub type StaticConsole = DstA64<dyn log::Log, 2080>;
static LOGGER: SyncUnsafeCell<Option<StaticConsole>> = SyncUnsafeCell::new(None);

pub struct ConsoleLoggerBuilder {
    logger: StaticConsole,
    max_level: log::LevelFilter,
}

impl ConsoleLoggerBuilder {
    pub fn new() -> Result<Self, LoggerError> {
        let peb = ProcessEnvironmentBlock::current_process();
        let handle = peb.standard_output().ok_or_else(|| LoggerError::NotInitialized)?;

        Ok(Self {
            logger: DstA64::new(ConsoleLogger::new(handle, ConsoleMode::Existing)),
            max_level: log::LevelFilter::Info,
        })
    }

    pub fn build(self) -> Result<&'static dyn log::Log, LoggerError> {

        unsafe { 
            let slot = LOGGER.get();
            *slot = Some(self.logger);  
            let logger = (*slot).as_ref().unwrap_unchecked().deref();
            log::set_logger(logger)?;
            log::set_max_level(self.max_level);

            Ok(logger)
        }
    }
}