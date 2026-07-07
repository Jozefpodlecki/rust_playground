use ntapi::ntpsapi::*;
use winapi::{ctypes::c_void, shared::{ntdef::{HANDLE, NTSTATUS, OBJECT_ATTRIBUTES}, ntstatus::STATUS_SUCCESS}, um::winnt::*};

use crate::ThreadError;

#[repr(align(16))]
pub struct AlignedContext(CONTEXT);

impl Default for AlignedContext {
    fn default() -> Self {
        Self(unsafe { core::mem::zeroed() })
    }
}

#[derive(Clone, Copy)]
pub struct ThreadHandle(*mut winapi::ctypes::c_void, ThreadOpenFlags);

unsafe impl Send for ThreadHandle {}
unsafe impl Sync for ThreadHandle {}

#[derive(Clone, Copy)]
pub struct ThreadOpenFlags(u32);

impl ThreadOpenFlags {
    pub const SUSPEND_RESUME: Self = Self(THREAD_SUSPEND_RESUME);
    pub const GET_CONTEXT: Self = Self(THREAD_GET_CONTEXT);
    pub const SET_CONTEXT: Self = Self(THREAD_SET_CONTEXT);
    pub const TERMINATE: Self = Self(THREAD_TERMINATE);

    pub const ALL: Self = Self(
        THREAD_SUSPEND_RESUME | THREAD_GET_CONTEXT | THREAD_SET_CONTEXT | THREAD_TERMINATE,
    );

    pub fn contains(&self, flags: Self) -> bool {
        (self.0 & flags.0) == flags.0
    }
}

#[derive(Debug)]
pub enum SystemError {
    NtStatus(NTSTATUS),
    Thread(ThreadError),
    InvalidFlags,
}

impl ThreadHandle {
    pub fn open(tid: *mut c_void, flags: ThreadOpenFlags) -> Result<Self, SystemError> {
        unsafe {
            
            let mut client_id = ntapi::ntapi_base::CLIENT_ID {
                UniqueProcess: 0 as *mut _,
                UniqueThread: tid as *mut _,
            };
            let mut thread_handle: HANDLE = core::ptr::null_mut();
            let mut attributes: OBJECT_ATTRIBUTES = core::mem::zeroed();

            let status = NtOpenThread(
                &mut thread_handle,
                flags.0,
                &mut attributes,
                &mut client_id,
            );

            if status != STATUS_SUCCESS {
                return Err(SystemError::NtStatus(status));
            }

            Ok(Self(thread_handle, flags))
        }
    }

    pub fn open_current(flags: ThreadOpenFlags) -> Result<Self, SystemError> {
        let tid = unsafe { NtCurrentThreadId() };
        Self::open(tid, flags)
    }

     pub fn suspend(&self) -> Result<u32, SystemError> {

        if !self.1.contains(ThreadOpenFlags::SUSPEND_RESUME) {
            return Err(SystemError::InvalidFlags);
        }

        let mut count = 0;
        let status = unsafe { NtSuspendThread(self.0, &mut count) };

        if status != STATUS_SUCCESS {
            return Err(SystemError::NtStatus(status));
        }

        Ok(count)
    }

    pub fn resume(&self) -> Result<u32, SystemError> {
        
        if !self.1.contains(ThreadOpenFlags::SUSPEND_RESUME) {
            return Err(SystemError::InvalidFlags);
        }

        let mut count = 0;
        let status = unsafe { NtResumeThread(self.0, &mut count) };

        if status != STATUS_SUCCESS {
            return Err(SystemError::NtStatus(status));
        }

        Ok(count)
    }

    pub fn alert_resume(&self) -> Result<u32, SystemError> {

        if !self.1.contains(ThreadOpenFlags::SUSPEND_RESUME) {
            return Err(SystemError::InvalidFlags);
        }

        let mut count = 0;
        let status = unsafe { NtAlertResumeThread(self.0, &mut count) };

        if status != STATUS_SUCCESS {
            return Err(SystemError::NtStatus(status));
        }

        Ok(count)
    }

    pub fn get_context(&self) -> Result<AlignedContext, SystemError> {

        if !self.1.contains(ThreadOpenFlags::GET_CONTEXT) {
            return Err(SystemError::InvalidFlags);
        }
        
        let mut context = AlignedContext::default();
        context.0.ContextFlags = CONTEXT_ALL;
        let status = unsafe { NtGetContextThread(self.0, &mut context.0) };
        
        if status != STATUS_SUCCESS {
            return Err(SystemError::NtStatus(status));
        }

        Ok(context)
    }

    pub fn set_context(&self, mut context: AlignedContext) -> Result<(), SystemError> {

        if !self.1.contains(ThreadOpenFlags::SET_CONTEXT) {
            return Err(SystemError::InvalidFlags);
        }

        let status = unsafe { NtSetContextThread(self.0, &mut context.0) };

        if status != STATUS_SUCCESS {
            return Err(SystemError::NtStatus(status));
        }

        Ok(())
    }

    pub fn terminate(&self, exit_code: i32) -> Result<(), SystemError> {
        
        if !self.1.contains(ThreadOpenFlags::TERMINATE) {
            return Err(SystemError::InvalidFlags);
        }

        let status = unsafe { NtTerminateThread(self.0, exit_code) };

        if status != STATUS_SUCCESS {
            return Err(SystemError::NtStatus(status));
        }

        Ok(())
    }

   
    pub fn handle(&self) -> HANDLE {
        self.0
    }
}

