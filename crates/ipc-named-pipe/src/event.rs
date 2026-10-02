use core::{mem, ptr::null_mut};

use log::info;
use ntapi::ntioapi::*;
use win_platform::{NtError, syscalls::{NtClose, NtCreateEvent, NtWaitForSingleObject, }, types::ObjectAttributes};
use winapi::{shared::ntdef::{NotificationEvent, OBJECT_ATTRIBUTES}, um::winnt::*};
use win_platform::types::HANDLE;

pub struct Event(HANDLE);

impl Event {
    pub fn new() -> Result<Self, NtError> {
        unsafe {
            let mut handle: HANDLE = null_mut();
            let mut attrs = ObjectAttributes::new();
            let mut attrs_raw = attrs.as_raw();

            NtCreateEvent(
                &mut handle,
                EVENT_ALL_ACCESS,
                &mut attrs_raw,
                NotificationEvent,
                0,
            )
            .ok()?;

            Ok(Self(handle))
        }
    }

    pub fn as_handle(&self) -> HANDLE {
        self.0
    }

    pub fn wait(&self) -> Result<(), NtError> {
        NtWaitForSingleObject(self.0, 0, null_mut()).ok()
    }
}

impl Drop for Event {
    fn drop(&mut self) {
        unsafe { NtClose(self.0) };
    }
}