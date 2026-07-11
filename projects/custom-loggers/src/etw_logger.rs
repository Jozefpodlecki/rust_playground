
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError, info};
use ntapi::ntdbg::REGHANDLE;
use toolkit::{U16CStackString, println};
use winapi::{ctypes::c_void, shared::{evntprov::{EVENT_DATA_DESCRIPTOR, EVENT_DESCRIPTOR, EventDataDescCreate, EventRegister, EventUnregister, EventWriteEx}, guiddef::GUID, minwindef::ULONG}, um::errhandlingapi::{GetLastError, SetLastError}};
use core::fmt::Write;


#[link(name = "advapi32")]
unsafe extern "system" {
    pub fn EtwEventWriteEx(
        RegHandle: REGHANDLE,
        EventDescriptor: *const EVENT_DESCRIPTOR,
        Filter: u64,
        Flags: u32,
        ActivityId: *const GUID,
        RelatedActivityId: *const GUID,
        UserDataCount: u32,
        UserData: *const EVENT_DATA_DESCRIPTOR,
    ) -> u32;
}

const MY_PROVIDER_GUID: GUID = GUID {
    Data1: 0x12345678,
    Data2: 0x1234,
    Data3: 0x1234,
    Data4: [0x12, 0x34, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC],
};

const EVENT_INFO: u16 = 1;
const EVENT_WARN: u16 = 2;
const EVENT_ERROR: u16 = 3;
const EVENT_DEBUG: u16 = 4;

static mut LOGGER: Option<EventViewerLogger> = None;

pub struct EventViewerLogger {
    reg_handle: REGHANDLE,
    level: Level,
}

impl EventViewerLogger {
    pub fn init() -> Result<(), SetLoggerError> {
        let logger = Self::new();
    
        unsafe {
            LOGGER = Some(logger);
            let logger_ref: &'static EventViewerLogger = &*LOGGER.as_ref().unwrap() as &'static _;
            log::set_logger(logger_ref)?;
        }
        
        log::set_max_level(LevelFilter::Debug);
        Ok(())
    }

    pub fn new() -> Self {
        let mut reg_handle = 0;
        
        unsafe {
            let status = EventRegister(
                &MY_PROVIDER_GUID,
                None,
                core::ptr::null_mut(),
                &mut reg_handle,
            );

            println!("EventRegister status {}", status);

            if status != 0 {
                panic!("Failed to register ETW provider: {}", status);
            }
        }
        
        Self {
            reg_handle,
            level: Level::Debug,
        }
    }
    
    fn write_event(
        &self,
        event_id: u16,
        level: u8,
        record: &Record,
    ) {
        if self.reg_handle == 0 {
            return;
        }
        
        let descriptor = EVENT_DESCRIPTOR {
            Id: event_id,
            Version: 0,
            Channel: 16,
            Level: level,
            Opcode: 0,
            Task: 0,
            Keyword: 0x8000000000000000,
        };
        
        unsafe {
            let mut message = U16CStackString::<260>::new();

            if let Err(err) = write!(message, "{}", record.args()) {
                panic!("{err}")
            }
            
            let mut data_desc: EVENT_DATA_DESCRIPTOR = core::mem::zeroed();
            EventDataDescCreate(
                &mut data_desc,
                message.as_ptr() as *const c_void,
                ((message.len() + 1) * 2) as ULONG,
            );

            let last_error = GetLastError();
            println!("last_error {}", last_error);
            SetLastError(0);
            
            let result = EventWriteEx(
                self.reg_handle,
                &descriptor,
                0,
                0,
                core::ptr::null(),
                core::ptr::null(),
                1,
                &mut data_desc,
            );

            let last_error = GetLastError();

            println!("{}", result);
            println!("last_error {}", last_error);
        }
    }
}

impl Drop for EventViewerLogger {
    fn drop(&mut self) {
        if self.reg_handle != 0 {
            unsafe {
                EventUnregister(self.reg_handle);
            }
        }
    }
}

impl log::Log for EventViewerLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }
    
    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        
        let (event_id, etw_level) = match record.level() {
            Level::Error => (EVENT_ERROR, 2),
            Level::Warn => (EVENT_WARN, 3),
            Level::Info => (EVENT_INFO, 4),
            Level::Debug => (EVENT_DEBUG, 5),
            Level::Trace => (EVENT_DEBUG, 5),
        };
        
        self.write_event(event_id, etw_level, &record);
    }
    
    fn flush(&self) {}
}