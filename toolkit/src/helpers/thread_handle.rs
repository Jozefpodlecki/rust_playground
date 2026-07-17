use core::{mem, ops::{Deref, DerefMut}};

use ntapi::ntpsapi::*;
use winapi::{ctypes::c_void, shared::{ntdef::{HANDLE, NTSTATUS, OBJECT_ATTRIBUTES}, ntstatus::STATUS_SUCCESS}, um::winnt::*};

use crate::ThreadError;

const THREAD_SUSPEND_COUNT: u32 = 35;
const THREAD_WAIT_REASON: u32 = 37;
const DELAY_EXECUTION_WAIT: u32 = 4;
const WR_DELAY_EXECUTION_WAIT: u32 = 11;

pub enum ThreadState {
    Active,
    Suspended,
    NtDelayExecution
}

#[repr(align(16))]
pub struct AlignedContext(CONTEXT);

impl Default for AlignedContext {
    fn default() -> Self {
        Self(unsafe { core::mem::zeroed() })
    }
}

impl Deref for AlignedContext {
    type Target = CONTEXT;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AlignedContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone, Copy)]
pub struct ThreadHandle(pub *mut winapi::ctypes::c_void, ThreadOpenFlags);

unsafe impl Send for ThreadHandle {}
unsafe impl Sync for ThreadHandle {}

#[derive(Clone, Copy)]
pub struct ThreadOpenFlags(u32);

impl ThreadOpenFlags {
    pub const SUSPEND_RESUME: Self = Self(THREAD_SUSPEND_RESUME);
    pub const GET_CONTEXT: Self = Self(THREAD_GET_CONTEXT);
    pub const SET_CONTEXT: Self = Self(THREAD_SET_CONTEXT);
    pub const TERMINATE: Self = Self(THREAD_TERMINATE);
    pub const QUERY_INFORMATION: Self = Self(THREAD_QUERY_INFORMATION);

    pub const ALL: Self = Self(
        THREAD_SUSPEND_RESUME 
        | THREAD_GET_CONTEXT 
        | THREAD_SET_CONTEXT 
        | THREAD_TERMINATE
        | THREAD_QUERY_INFORMATION
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
        // context.0.ContextFlags = CONTEXT_ALL;
        context.0.ContextFlags = CONTEXT_FULL;
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

     pub fn get_state(&self) -> Result<ThreadState, SystemError> {
        let mut suspend_count: u32 = 0;
        let status = unsafe {
            NtQueryInformationThread(
                self.0,
                ThreadSuspendCount,
                &mut suspend_count as *mut _ as *mut _,
                mem::size_of::<u32>() as u32,
                core::ptr::null_mut(),
            )
        };

        if status != STATUS_SUCCESS {
            return Err(SystemError::NtStatus(status));
        }

        if suspend_count > 0 {
            return Ok(ThreadState::Suspended);
        }

        let mut wait_reason: u32 = 0;
        let status = unsafe {
            NtQueryInformationThread(
                self.0,
                THREAD_WAIT_REASON,
                &mut wait_reason as *mut _ as *mut _,
                mem::size_of::<u32>() as u32,
                core::ptr::null_mut(),
            )
        };

        if status != STATUS_SUCCESS {
            return Err(SystemError::NtStatus(status));
        }

        if wait_reason == DELAY_EXECUTION_WAIT || wait_reason == WR_DELAY_EXECUTION_WAIT {
            return Ok(ThreadState::NtDelayExecution);
        }

        Ok(ThreadState::Active)
    }

    pub fn terminate(self, exit_code: i32) -> Result<(), SystemError> {
        
        if !self.1.contains(ThreadOpenFlags::TERMINATE) {
            return Err(SystemError::InvalidFlags);
        }

        let status = unsafe { NtTerminateThread(self.0, exit_code) };

        if status != STATUS_SUCCESS {
            return Err(SystemError::NtStatus(status));
        }

        Ok(())
    }

    pub fn tid(&self) -> usize {
        unsafe {
            let mut basic_info: THREAD_BASIC_INFORMATION = core::mem::zeroed();
            let mut return_length: u32 = 0;

            let status = NtQueryInformationThread(
                self.0,
                ThreadBasicInformation,
                &mut basic_info as *mut _ as *mut _,
                core::mem::size_of::<THREAD_BASIC_INFORMATION>() as u32,
                &mut return_length,
            );

            if status != STATUS_SUCCESS {
                return 0;
            }

            basic_info.ClientId.UniqueThread as usize
        }
    }
   
    pub fn handle(&self) -> HANDLE {
        self.0
    }
}

