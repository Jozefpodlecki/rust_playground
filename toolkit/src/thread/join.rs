use core::{ptr, sync::atomic::{AtomicBool, Ordering}};
use ntapi::ntobapi::{NtClose, NtWaitForSingleObject};
use winapi::shared::ntdef::{HANDLE, NTSTATUS};
use winapi::shared::ntstatus::STATUS_SUCCESS;
use winapi::um::winnt::{LARGE_INTEGER, THREAD_ALL_ACCESS};
use crate::*;
use crate::arc::Arc;

pub struct JoinHandle<T> {
    pub handle: HANDLE,
    pub packet: Packet<T>,
}

impl<T> JoinHandle<T> {
    pub fn is_finished(&self) -> bool {
        
        let status = unsafe {
            let mut delay: LARGE_INTEGER = core::mem::zeroed();
            *delay.QuadPart_mut() = 0;
            NtWaitForSingleObject(
                self.handle,
                0,
                &mut delay,
            )
        };
        status == STATUS_SUCCESS
    }

    pub fn join(self) -> Result<T, ThreadError> {
        unsafe {
            let status = NtWaitForSingleObject(
                self.handle,
                0,
                ptr::null_mut(),
            );
            
            if status != STATUS_SUCCESS {
                return Err(ThreadError::WaitFailed(status));
            }
            
            NtClose(self.handle);
            
            self.packet.take_result().ok_or(ThreadError::NoResult)
        }
    }

    pub fn join_timeout(self, timeout_ms: u64) -> Result<T, ThreadError> {
        if self.handle.is_null() {
            return Err(ThreadError::InvalidHandle);
        }

        let status = unsafe {
            let mut delay: LARGE_INTEGER = core::mem::zeroed();
            *delay.QuadPart_mut() = -(timeout_ms as i64 * 10_000);
            NtWaitForSingleObject(
                self.handle,
                0,
                &mut delay,
            )
        };

        match status {
            STATUS_SUCCESS => {
                unsafe {
                    let _ = NtClose(self.handle);
                }
                self.packet.take_result().ok_or(ThreadError::NoResult)
            }
            status if status == 0x102 => Err(ThreadError::Timeout),
            _ => Err(ThreadError::WaitFailed(status)),
        }
    }
}