#![allow(non_upper_case_globals)]

pub type HANDLE = *mut core::ffi::c_void;
pub type PHANDLE = *mut HANDLE;
pub type PVOID = *mut core::ffi::c_void;
pub type PCVOID = *const core::ffi::c_void;
pub type VOID = core::ffi::c_void;

pub const NtCurrentProcess: HANDLE = -1isize as HANDLE;
pub const NtCurrentThread: HANDLE = -2isize as HANDLE;
pub const NtCurrentSession: HANDLE = -3isize as HANDLE;