use core::ptr::null_mut;
use core::{mem, ptr};
use ntapi::ntapi_base::CLIENT_ID;
use ntapi::ntpsapi::{NtCurrentProcess, THREAD_BASIC_INFORMATION, THREAD_CREATE_FLAGS_CREATE_SUSPENDED, ThreadBasicInformation};
use ntapi::ntrtl::RtlExitUserThread;
use winapi::shared::{ntdef::HANDLE, ntstatus::STATUS_SUCCESS};
use winapi::um::winnt::THREAD_ALL_ACCESS;

use crate::syscalls::{NtCreateThreadEx, NtQueryInformationThread};
use crate::{Sleeper, ThreadError, println};

pub struct SuspendedThread(HANDLE);

unsafe extern "system" fn suspended_thread_entry(param: *mut winapi::ctypes::c_void) {
    loop {
        // println!("SuspendedThread");
        Sleeper::sleep(1000);
    }
}

impl SuspendedThread {
    pub fn new() -> Result<Self, ThreadError> {
        let mut handle: HANDLE = ptr::null_mut();

        let status = unsafe {
            NtCreateThreadEx(
                &mut handle,
                THREAD_ALL_ACCESS,
                ptr::null_mut(),
                NtCurrentProcess,
                suspended_thread_entry as *mut _,
                // null_mut(),
                null_mut(),
                THREAD_CREATE_FLAGS_CREATE_SUSPENDED,
                0,
                0,
                0,
                ptr::null_mut(),
            )
        };

        if status != STATUS_SUCCESS {
            return Err(ThreadError::CreationFailed(status));
        }

        Ok(Self(handle))
    }

    pub fn tid(&self) -> Result<usize, ThreadError> {
        let mut info: THREAD_BASIC_INFORMATION = unsafe { core::mem::zeroed() };
        let mut return_length = 0;

        let status = unsafe {
            NtQueryInformationThread(
                self.0,
                ThreadBasicInformation,
                &mut info as *mut _ as *mut _,
                core::mem::size_of::<THREAD_BASIC_INFORMATION>() as u32,
                &mut return_length,
            )
        };

        if status != STATUS_SUCCESS {
            return Err(ThreadError::QueryFailed(status));
        }

        Ok(info.ClientId.UniqueThread as usize)
    }

    pub fn into_handle(self) -> HANDLE {
        let handle = self.0;
        core::mem::forget(self);
        handle
    }
}