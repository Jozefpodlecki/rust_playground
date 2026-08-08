use core::{fmt, time::Duration};

use toolkit::{Sleeper, U16CStackString, println, syscalls::NtClose};
use winapi::{shared::minwindef::FALSE, um::{errhandlingapi::GetLastError, memoryapi::{FILE_MAP_ALL_ACCESS, MapViewOfFile, OpenFileMappingW}, synchapi::{OpenEventW, OpenMutexW, WaitForSingleObject}}};

use crate::{error::{ClientError, SharedMemoryError}, shared::{MUTEX_ALL_ACCESS, SharedMemory}};

pub struct Client {
    shared_memory: SharedMemory,
    _mapping_handle: *mut winapi::ctypes::c_void,
}

impl Client {
    fn open_file_mapping() -> Result<*mut winapi::ctypes::c_void, ClientError> {
        unsafe {
            let name = U16CStackString::<100>::from_str("Local\\SharedMemory").unwrap();
            let handle = OpenFileMappingW(FILE_MAP_ALL_ACCESS, FALSE, name.as_ptr());
            if handle.is_null() {
                let error = GetLastError();
                Err(ClientError::OpenFileMappingFailed(error))
            } else {
                Ok(handle)
            }
        }
    }

    fn map_view(handle: *mut winapi::ctypes::c_void) -> Result<*mut winapi::ctypes::c_void, ClientError> {
        unsafe {
            let view = MapViewOfFile(
                handle,
                FILE_MAP_ALL_ACCESS,
                0,
                0,
                crate::shared::SHARED_MEMORY_SIZE as usize,
            );
            if view.is_null() {
                NtClose(handle);
                Err(ClientError::MapViewOfFileFailed)
            } else {
                Ok(view)
            }
        }
    }

    fn open_mutex() -> Result<*mut winapi::ctypes::c_void, ClientError> {
        unsafe {
            let name = U16CStackString::<100>::from_str("Global\\SharedMemoryMutex").unwrap();
            let handle = OpenMutexW(MUTEX_ALL_ACCESS, FALSE, name.as_ptr());
            if handle.is_null() {
                Err(ClientError::OpenMutexFailed)
            } else {
                Ok(handle)
            }
        }
    }

    fn open_event() -> Result<*mut winapi::ctypes::c_void, ClientError> {
        unsafe {
            let name = U16CStackString::<100>::from_str("Global\\SharedMemoryEvent").unwrap();
            let handle = OpenEventW(winapi::um::winnt::SYNCHRONIZE, FALSE, name.as_ptr());
            if handle.is_null() {
                Err(ClientError::OpenEventFailed)
            } else {
                Ok(handle)
            }
        }
    }

    pub fn new() -> Result<Self, ClientError> {
        let mapping_handle = Self::open_file_mapping()?;
        let view_ptr = Self::map_view(mapping_handle)?;
        let mutex_handle = Self::open_mutex()?;
        let event_handle = Self::open_event()?;

        let shared_memory = SharedMemory::new(view_ptr, mutex_handle, event_handle);

        Ok(Self {
            shared_memory,
            _mapping_handle: mapping_handle,
        })
    }

    pub fn run(&self) -> Result<(), ClientError> {
        println!("Launching client runner");

        loop {
            Sleeper::sleep(1000);
            match self.shared_memory.wait_for_signal(Duration::from_secs(5)) {
                Ok(()) => {
                    self.shared_memory.lock()?;

                    match self.shared_memory.read_message() {
                        Ok(msg) => {
                            if !msg.is_empty() {
                                println!("Client read: {} (timestamp: {:?})", msg, self.shared_memory.timestamp());
                            }
                        }
                        Err(err) => {
                            self.shared_memory.unlock()?;
                            return Err(ClientError::SharedMemory(err));
                        }
                    }

                    self.shared_memory.unlock()?;
                }
                Err(SharedMemoryError::WaitTimeout) => {
                    println!("Client: Timeout waiting for data");
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.shared_memory.close();
    }
}