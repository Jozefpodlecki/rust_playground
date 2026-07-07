use core::{mem, ptr};
use ntapi::ntapi_base::CLIENT_ID;
use ntapi::ntpsapi::{NtCreateThread, NtCreateThreadEx, NtCurrentProcess};
use ntapi::ntrtl::RtlExitUserThread;
use winapi::shared::{ntdef::HANDLE, ntstatus::STATUS_SUCCESS};
use winapi::um::winnt::THREAD_ALL_ACCESS;
use crate::*;
use crate::arc::Arc;

pub struct Thread;

impl Thread {

    pub fn spawn<F, T>(func: F) -> Result<JoinHandle<T>, ThreadError>
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        let packet = Packet::new(func);

        unsafe extern "system" fn thread_entry<T>(param: *mut winapi::ctypes::c_void) {
            
            let packet = Packet::<T>::from_ptr(param as _);
            let result = packet.execute();
            packet.set_result(result);
            
            unsafe { RtlExitUserThread(0) };
        }

        let packet_clone = packet.into_raw();
        let param_ptr: *mut winapi::ctypes::c_void = packet_clone as _;
        let mut thread_handle: HANDLE = ptr::null_mut();
        let mut context = ThreadContext::new();
        let stack_size = 1024 * 1024;
        let stack = ThreadStack::new(stack_size).unwrap();
        let mut initial_teb = stack.to_initial_teb();
        let rsp = stack.rsp();

        let rip = thread_entry::<T> as _;
        let rcx = param_ptr as u64;
        context.set_thread_context(rsp, rip, rcx);

        let mut client_id: CLIENT_ID = unsafe { mem::zeroed() };

        let status = unsafe {
            NtCreateThread(
                &mut thread_handle as _,
                THREAD_ALL_ACCESS,
                ptr::null_mut(),
                NtCurrentProcess,
                &mut client_id as _,
                &mut *context,
                &mut initial_teb as _,
                0,
            )
        };
        
        if status != STATUS_SUCCESS {
            stack.free().unwrap();
            return Err(ThreadError::CreationFailed(status));
        }

        Ok(JoinHandle {
            handle: thread_handle,
            packet,
            stack: Some(stack)
        })
    }

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

        let packet_clone = packet.into_raw();
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
            stack: None,
        })
    }
}