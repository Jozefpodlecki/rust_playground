
use core::{mem::{self, zeroed}, ptr::null_mut};

use alloc::string::String;
use ntapi::{ntexapi::{NtCreateEvent, NtResetEvent, NtSetEvent}, ntobapi::NtWaitForSingleObject, ntrtl::{RtlCreateUnicodeString, RtlInitUnicodeString}};
use toolkit::{U16CStackString, println};
use winapi::{shared::ntdef::{NTSTATUS, OBJ_CASE_INSENSITIVE, OBJ_OPENIF, OBJECT_ATTRIBUTES, UNICODE_STRING}, um::{synchapi::{CREATE_EVENT_MANUAL_RESET, CreateEventExW}, winnt::{EVENT_MODIFY_STATE, SYNCHRONIZE}}};

#[derive(Clone, Copy)]
pub struct EventHandle(*mut winapi::ctypes::c_void);

unsafe impl Send for EventHandle {}
unsafe impl Sync for EventHandle {}

pub enum EventType {
    // Caller decide about state of event
    NotificationEvent,
    SynchronizationEvent
}

impl EventHandle {
    pub fn auto(name: &U16CStackString<50>) -> Self {
        let handle = unsafe { 
            CreateEventExW(null_mut(), name.as_ptr(), 0, EVENT_MODIFY_STATE | SYNCHRONIZE)
        };
        EventHandle(handle)
    }

    pub fn manual(name: &U16CStackString<50>) -> Result<Self, NTSTATUS> {
        unsafe { 
            let mut handle: *mut _ = null_mut();
            let mut uc_str: UNICODE_STRING = zeroed();
            RtlInitUnicodeString(&mut uc_str, name.as_ptr());
            let mut obj_attr = OBJECT_ATTRIBUTES {
                Length: mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
                RootDirectory: 0x80 as _,
                ObjectName: &mut uc_str,
                Attributes: OBJ_OPENIF,
                SecurityDescriptor: null_mut(),
                SecurityQualityOfService: null_mut(),
            };


             let status = NtCreateEvent(
                &mut handle,
                EVENT_MODIFY_STATE | SYNCHRONIZE,
                &mut obj_attr,
                0,
                0);
            // pub const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000000D;
            // C0000024 STATUS_OBJECT_TYPE_MISMATCH: NTSTATUS = 0xC0000024;
            if status == 0 {
                Ok(Self(handle))
            } else {
                Err(status)
            }
        }

        // let handle = unsafe { 
        //     CreateEventExW(null_mut(), name.as_ptr(), CREATE_EVENT_MANUAL_RESET, EVENT_MODIFY_STATE | SYNCHRONIZE)
        // };

        //  Ok(Self(handle))
    }

    pub fn wait(&self) -> Result<(), NTSTATUS> {
        let status = unsafe { NtWaitForSingleObject(self.0, 1, null_mut()) };
        if status == 0 {
            Ok(())
        } else {
            Err(status)
        }
    }

    pub fn set(&self) -> Result<(), NTSTATUS> {
        let status = unsafe { NtSetEvent(self.0, null_mut()) };
        if status == 0 {
            Ok(())
        } else {
            Err(status)
        }
    }

    pub fn reset(&self) -> Result<(), NTSTATUS> {
        let status = unsafe { NtResetEvent(self.0, null_mut()) };
        if status == 0 {
            Ok(())
        } else {
            Err(status)
        }
    }
}