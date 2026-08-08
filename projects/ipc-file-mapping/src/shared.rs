use core::{fmt, mem::{self, zeroed}, ptr::null_mut, time::Duration};

use toolkit::{println, syscalls::NtWaitForSingleObject};
use winapi::um::{
    synchapi::{ReleaseMutex, SetEvent, WaitForSingleObject}, winnt::LARGE_INTEGER,
};
use ntapi::{ntexapi::{NtReleaseMutant, NtSetEvent}, ntobapi::NtClose};
use winapi::um::memoryapi::UnmapViewOfFile;

use crate::{error::SharedMemoryError, types::{KUserSharedData, SystemTime}};

pub const SHARED_MEMORY_SIZE: u32 = 1024;
pub const MUTEX_ALL_ACCESS: u32 = 0x001F0001;
pub const MAX_MESSAGE_SIZE: usize = 256;
pub const WAIT_TIMEOUT: u32 = 0x102;

#[repr(C)]
pub struct SharedMemoryInner {
    pub kind: u32,
    pub timestamp: SystemTime,
    pub data_len: u32,
    pub data: [u8; MAX_MESSAGE_SIZE],
}

impl SharedMemoryInner {
    pub fn as_mut(view_ptr: *mut winapi::ctypes::c_void) -> &'static mut Self {
        unsafe { &mut *(view_ptr as *mut SharedMemoryInner) }
    }
}

impl SharedMemoryInner {
    pub fn new() -> Self {
        Self {
            kind: 0,
            timestamp: SystemTime::default(),
            data_len: 0,
            data: [0u8; MAX_MESSAGE_SIZE],
        }
    }
}

pub struct SharedMemory {
    kuser: &'static KUserSharedData,
    inner: &'static mut SharedMemoryInner,
    mutex_handle: *mut winapi::ctypes::c_void,
    event_handle: *mut winapi::ctypes::c_void,
}

impl SharedMemory {
    pub fn new(
        view_ptr: *mut winapi::ctypes::c_void,
        mutex_handle: *mut winapi::ctypes::c_void,
        event_handle: *mut winapi::ctypes::c_void,
    ) -> Self {
        let inner = unsafe { &mut *(view_ptr as *mut SharedMemoryInner) };

        Self {
            kuser: KUserSharedData::new(),
            inner,
            mutex_handle,
            event_handle,
        }
    }

    pub fn init(&mut self) {
        *self.inner = SharedMemoryInner::new();
    }

    pub fn lock(&self) -> Result<(), SharedMemoryError> {
        self.lock_with_timeout(Duration::MAX)
    }

    pub fn lock_with_timeout(&self, timeout: Duration) -> Result<(), SharedMemoryError> {
        unsafe {
            let timeout_ms = timeout.as_millis();
            let mut timeout_nt: LARGE_INTEGER = mem::zeroed();
            *timeout_nt.QuadPart_mut() = -(timeout_ms as i64) * 10_000;
            
            let status = NtWaitForSingleObject(self.mutex_handle, 0, &mut timeout_nt);
            
            if status == 0 {
                Ok(())
            } else if status == WAIT_TIMEOUT as i32 {
                Err(SharedMemoryError::LockTimeout)
            } else {
                Err(SharedMemoryError::LockFailed)
            }
        }
    }

    pub fn unlock(&self) -> Result<(), SharedMemoryError> {
        unsafe {
            let status = NtReleaseMutant(self.mutex_handle, null_mut());
            if status == 0 {
                Ok(())
            } else {
                Err(SharedMemoryError::LockFailed)
            }
        }
    }

    pub fn signal(&self) -> Result<(), SharedMemoryError> {
        unsafe {
            let status = NtSetEvent(self.event_handle, null_mut());
            if status == 0 {
                Ok(())
            } else {
                Err(SharedMemoryError::LockFailed)
            }
        }
    }

    pub fn timestamp(&self) -> SystemTime {
        self.inner.timestamp
    }

    pub fn write_message(&mut self, msg: &str) -> Result<(), SharedMemoryError> {
        let bytes = msg.as_bytes();
        if bytes.len() > MAX_MESSAGE_SIZE {
            return Err(SharedMemoryError::MessageTooLong);
        }

        self.inner.timestamp = self.kuser.system_time();
        self.inner.data[..bytes.len()].copy_from_slice(bytes);
        self.inner.data_len = bytes.len() as u32;
        self.inner.kind = 1;

        Ok(())
    }

    pub fn read_message(&self) -> Result<&str, SharedMemoryError> {
        if self.inner.data_len == 0 {
            return Ok("");
        }

        let data = &self.inner.data[..self.inner.data_len as usize];
        core::str::from_utf8(data).map_err(|_| SharedMemoryError::InvalidUtf8)
    }

    pub fn read_message_bytes(&self) -> &[u8] {
        &self.inner.data[..self.inner.data_len as usize]
    }

    pub fn clear(&mut self) {
        self.inner.data_len = 0;
        self.inner.kind = 0;
    }

    pub fn view_ptr(&mut self) -> *mut winapi::ctypes::c_void {
        self.inner as *mut _ as *mut winapi::ctypes::c_void
    }

    pub fn wait_for_signal(&self, timeout: Duration) -> Result<(), SharedMemoryError> {
        unsafe {
            let timeout_ms = timeout.as_millis();
            let mut timeout_nt: LARGE_INTEGER = zeroed();
            *timeout_nt.QuadPart_mut() = -(timeout_ms as i64) * 10_000;
            
            let status = NtWaitForSingleObject(self.event_handle, 0, &mut timeout_nt);
            
            if status == 0 {
                Ok(())
            } else if status == WAIT_TIMEOUT as i32 {
                Err(SharedMemoryError::WaitTimeout)
            } else {
                Err(SharedMemoryError::WaitFailed)
            }
        }
    }

    pub fn close(&mut self) {
        unsafe {
            if !self.mutex_handle.is_null() {
                ReleaseMutex(self.mutex_handle);
                NtClose(self.mutex_handle);
                self.mutex_handle = core::ptr::null_mut();
            }

            if !self.event_handle.is_null() {
                NtClose(self.event_handle);
                self.event_handle = core::ptr::null_mut();
            }
            
            if !self.view_ptr().is_null() {
                UnmapViewOfFile(self.view_ptr());
            }
        }
    }
}

const _: () = assert!(core::mem::size_of::<SharedMemoryInner>() <= SHARED_MEMORY_SIZE as usize);