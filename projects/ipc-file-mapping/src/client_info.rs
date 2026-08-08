use core::{mem, ptr, time::Duration};

use ntapi::{ntobapi::{NtClose, NtWaitForSingleObject}, ntpsapi::NtCurrentProcess};
use toolkit::{Sleeper, print, println, syscalls::NtCreateThreadEx};
use winapi::{shared::{minwindef::DWORD, ntdef::{HANDLE, NTSTATUS}, ntstatus::{STATUS_SUCCESS, STATUS_TIMEOUT}}, um::{errhandlingapi::GetLastError, fileapi::ReadFile, minwinbase::STILL_ACTIVE, processthreadsapi::PROCESS_INFORMATION, winnt::{LARGE_INTEGER, THREAD_ALL_ACCESS}}};

use crate::error::SharedMemoryError;

#[derive(Debug, PartialEq)]
pub enum ClientStatus {
    Alive,
    Dead,
    Unknown,
}

pub struct ClientInfo {
    pub process_handle: *mut winapi::ctypes::c_void,
    pub process_id: u32,
    pub stdout_read: HANDLE
}

impl ClientInfo {
    
    extern "system" fn logger_entry(stdout_read: *mut winapi::ctypes::c_void) {
        unsafe {
            println!(" logger_entry {:p}", stdout_read);

            loop {
                let mut buffer = [0; 1024];
                let mut bytes_read: DWORD = 0;
                let result = ReadFile(
                    stdout_read,
                    buffer.as_mut_ptr() as _,
                    buffer.len() as u32,
                    &mut bytes_read,
                    ptr::null_mut(),
                );

                if result == 0 {
                    
                    let error = GetLastError();
                    println!("ReadFile Failed error {error}");
                    if error == 0xE8 {
                        break;
                    }
                    break;
                }

                if bytes_read > 0 {
                    if let Ok(output) = core::str::from_utf8(&buffer[..bytes_read as usize]) {
                        print!("{}", output);
                    }
                }

                Sleeper::sleep(1000);
            }
        }
    }

    pub fn spawn_logger(&self) -> Result<HANDLE, NTSTATUS> {
        let mut handle: HANDLE = ptr::null_mut();

        let status = unsafe {
            NtCreateThreadEx(
                &mut handle,
                THREAD_ALL_ACCESS,
                ptr::null_mut(),
                NtCurrentProcess,
                Self::logger_entry as *mut _,
                self.stdout_read,
                0,
                0,
                0,
                0,
                ptr::null_mut(),
            )
        };

        if status != STATUS_SUCCESS {
            return Err(status);
        }

        Ok(handle)
    }

    pub fn check_status(&self) -> ClientStatus {
        unsafe {
            let mut exit_code: u32 = 0;
            let result = winapi::um::processthreadsapi::GetExitCodeProcess(
                self.process_handle,
                &mut exit_code,
            );

            if result == 0 {
                return ClientStatus::Unknown;
            }

            if exit_code != STILL_ACTIVE {
                return ClientStatus::Dead;
            }

            ClientStatus::Alive
        }
    }

    pub fn is_alive(&self) -> bool {
        self.check_status() == ClientStatus::Alive
    }

    pub fn wait_for_process(&self, timeout: Duration) -> Result<(), SharedMemoryError> {
        unsafe {
            let timeout_ms = timeout.as_millis();
            let mut timeout_nt: LARGE_INTEGER = mem::zeroed();
            *timeout_nt.QuadPart_mut() = -(timeout_ms as i64) * 10_000;
            
            let status = NtWaitForSingleObject(
                self.process_handle,
                0,
                &mut timeout_nt,
            );
            
            if status == 0 {
                Ok(())
            } else if status == STATUS_TIMEOUT as i32 {
                Err(SharedMemoryError::WaitTimeout)
            } else {
                Err(SharedMemoryError::WaitFailed)
            }
        }
    }
}

impl Drop for ClientInfo {
    fn drop(&mut self) {
        unsafe {
            if !self.process_handle.is_null() {
                NtClose(self.process_handle);
            }
        }
    }
}