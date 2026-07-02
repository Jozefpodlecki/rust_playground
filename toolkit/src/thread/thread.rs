use core::ptr;
use ntapi::ntpsapi::{NtCreateThreadEx, NtCurrentProcess};
use winapi::shared::{ntdef::HANDLE, ntstatus::STATUS_SUCCESS};
use winapi::um::winnt::THREAD_ALL_ACCESS;
use crate::*;
use crate::arc::Arc;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

pub struct Thread;

impl Thread {

    pub fn spawn_ex<F, T>(func: F) -> Result<JoinHandle<T>, ThreadError>
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        let packet = Packet::new(func);
        
        unsafe extern "system" fn thread_entry<T>(param: *mut winapi::ctypes::c_void) -> u32 {
            let packet = Packet::<T>::from_ptr(param as _);
            let result = packet.execute();
            packet.set_result(result);
            
            0
        }

        let packet_clone = packet.leak();
        let param_ptr: *mut winapi::ctypes::c_void = packet_clone as _;
        let mut handle: HANDLE = ptr::null_mut();

        let status = unsafe {
            NtCreateThreadEx(
                &mut handle,
                THREAD_ALL_ACCESS,
                ptr::null_mut(),
                NtCurrentProcess,
                thread_entry::<T> as *mut _,
                param_ptr,
                0,
                0,
                0,
                0,
                ptr::null_mut(),
            )
        };

        if status != STATUS_SUCCESS {
            packet_clone.drop();
            return Err(ThreadError::CreationFailed(status));
        }

        Ok(JoinHandle {
            handle,
            packet,
        })
    }
}