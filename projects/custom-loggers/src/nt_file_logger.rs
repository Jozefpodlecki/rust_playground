use log::{LevelFilter, Log, Metadata, Record};
use ntapi::ntexapi::KUSER_SHARED_DATA;
use toolkit::{BufWriter, File, FileError, Mutex, ProcessEnvironmentBlock, U16CStackString, Write as IoWrite, canonicalize, println};
use winapi::shared::ntdef::UNICODE_STRING;
use core::fmt::{self, Write};

use crate::time::SystemTime;

#[derive(Debug)]
pub enum LoggerError {
    InvalidPath,
    Canonicalize,
    FileCreate(FileError),
    SetLogger,
    Formatter(core::fmt::Error),
    Write,
}

impl fmt::Display for LoggerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoggerError::InvalidPath => write!(f, "invalid path"),
            LoggerError::Canonicalize => write!(f, "failed to canonicalize path"),
            LoggerError::FileCreate(_) => write!(f, "failed to create file"),
            LoggerError::SetLogger => write!(f, "failed to set logger"),
            LoggerError::Formatter(err) => write!(f, "formatting error: {}", err),
            LoggerError::Write => write!(f, "failed to write to file"),
        }
    }
}

static mut LOGGER: Option<NtFileLogger> = None;
static mut BUFFER: [u8; 1024] = [0; 1024];

struct Formatter {
    buf: &'static mut [u8],
    pos: usize,
}

impl Formatter {
    const fn new() -> Self {
        unsafe { BUFFER = core::mem::zeroed() };
        Self {
            buf: unsafe { BUFFER.as_mut_slice() },
            pos: 0,
        }
    }

    fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.pos]
    }
}

impl core::fmt::Write for Formatter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let remaining = self.buf.len() - self.pos;
        if bytes.len() > remaining {
            return Err(core::fmt::Error);
        }
        self.buf[self.pos..self.pos + bytes.len()].copy_from_slice(bytes);
        self.pos += bytes.len();
        Ok(())
    }
}

pub struct NtFileLogger {
    level: LevelFilter,
    writer: Mutex<BufWriter<File>>
}

unsafe impl Send for NtFileLogger {}
unsafe impl Sync for NtFileLogger {}

impl NtFileLogger {
    pub fn init(level: LevelFilter) -> Result<NtFileLoggerGuard<'static>, LoggerError> {
        let peb = ProcessEnvironmentBlock::current_process();
        let path = peb.executable_path();
        let parent_dir = path.parent();
        let file_name = path.file_stem();
        let system_time = SystemTime::now();
        
        let mut target_path = U16CStackString::<260>::new();
        write!(&mut target_path, "\\??\\{}\\{}_{}.log", parent_dir.display::<120>(), file_name.display::<120>(), system_time.to_hh_mm_ss());
        let file = File::create(target_path).map_err(LoggerError::FileCreate)?;

        let logger = NtFileLogger {
            level,
            writer: Mutex::new(BufWriter::new(file))
        };

        unsafe {
            LOGGER = Some(logger);
            let logger_ref: &'static NtFileLogger = &*(&*LOGGER.as_ref().unwrap() as *const _);
            log::set_logger(logger_ref).map_err(|_| LoggerError::SetLogger)?;
            log::set_max_level(level);
            Ok(NtFileLoggerGuard(logger_ref))
        }
    }

    fn log_inner(&self, record: &Record) -> Result<(), LoggerError> {
        if !self.enabled(record.metadata()) {
            return Ok(());
        }

        let system_time = SystemTime::now();
        let message = Self::format_message(record, system_time).map_err(LoggerError::Formatter)?;;
        println!("after message");
        let mut guard = self.writer.lock();
        guard.write_all(message.as_bytes()).map_err(|_| LoggerError::Write)?;
        guard.write_all(b"\n").map_err(|_| LoggerError::Write)?;
        Ok(())
    }

    fn format_message(record: &Record, system_time: SystemTime) -> Result<Formatter, core::fmt::Error> {
        let mut f = Formatter::new();
        write!(f, "{} [{}] [{}]", system_time, record.level(), record.target())?;
        
        if let Some(file) = record.file() {
            write!(f, " [{}", file)?;
            if let Some(line) = record.line() {
                write!(f, ":{}]", line)?;
            } else {
                write!(f, "]")?;
            }
        }
        
        write!(f, " {}", record.args())?;
        Ok(f)
    }
}

impl log::Log for NtFileLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if let Err(err) = self.log_inner(record) {
            panic!("{err}");
        }
    }

    fn flush(&self) {
        let mut guard = self.writer.lock();
        let _ = guard.flush_buf();
    }
}

pub struct NtFileLoggerGuard<'a>(&'a NtFileLogger);

impl<'a> Drop for NtFileLoggerGuard<'a> {
    fn drop(&mut self) {
        self.0.flush();
    }
}