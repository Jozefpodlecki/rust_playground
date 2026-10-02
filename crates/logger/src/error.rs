use core::fmt;

use log::SetLoggerError;
use win_platform::{NtError, io::FileError};

#[derive(Debug)]
pub enum LoggerError {
    AlreadyInitialized,
    NotInitialized,
    ProcessDoestNotHaveConsole,
    Io(FileError),
    AttachConsole(NtError),
    IoConsole(NtError),
    Fmt(core::fmt::Error),
}

impl fmt::Display for LoggerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyInitialized => write!(f, "logger already initialized"),
            Self::NotInitialized        => write!(f, "logger not initialized"),
            Self::ProcessDoestNotHaveConsole        => write!(f, "process does not have console"),
            Self::Io(err)                 => write!(f, "logger I/O error: {err}"),
            Self::AttachConsole(err)                 => write!(f, "could not attach console: {err}"),
            Self::IoConsole(err)                 => write!(f, "logger I/O error: {err}"),
            Self::Fmt(err)                => write!(f, "logger format error: {err}"),
        }
    }
}

impl From<SetLoggerError> for LoggerError {
    fn from(_err: SetLoggerError) -> Self { Self::AlreadyInitialized }
}

impl From<FileError> for LoggerError {
    fn from(err: FileError) -> Self { Self::Io(err) }
}

impl From<core::fmt::Error> for LoggerError {
    fn from(err: core::fmt::Error) -> Self { Self::Fmt(err) }
}

impl From<core::io::Error> for LoggerError {
    fn from(err: core::io::Error) -> Self {
        Self::Io(FileError::from_core_io(err))
    }
}

impl core::error::Error for LoggerError {}