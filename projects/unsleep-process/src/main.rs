#![no_std]
#![no_main]
#![windows_subsystem = "console"]

use core::panic::PanicInfo;

use ntapi::{ntexapi::NtDelayExecution, ntpsapi::{NtAlertResumeThread, NtCurrentThreadId, NtOpenThread}};
use utils::Thread;
use winapi::{shared::ntdef::{HANDLE, OBJECT_ATTRIBUTES}, um::winnt::{LARGE_INTEGER, THREAD_SUSPEND_RESUME}};

#[global_allocator]
static ALLOCATOR: emballoc::Allocator<8192> = emballoc::Allocator::new();


#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[derive(Clone, Copy)]
struct ThreadHandle(*mut winapi::ctypes::c_void);

unsafe impl Send for ThreadHandle {}
unsafe impl Sync for ThreadHandle {}


#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {

    let main_thread = unsafe {
        let tid = NtCurrentThreadId();
        let mut client_id = ntapi::ntapi_base::CLIENT_ID {
            UniqueProcess: 0 as *mut _,
            UniqueThread: tid as *mut _,
        };
        let mut thread_handle: HANDLE = core::ptr::null_mut();
        let mut attributes: OBJECT_ATTRIBUTES = core::mem::zeroed();

        let status = NtOpenThread(
            &mut thread_handle,
            THREAD_SUSPEND_RESUME,
            &mut attributes,
            &mut client_id,
        );

        ThreadHandle(thread_handle)
    };

    let handle = Thread::spawn_ex(move || {
        let main_thread = main_thread;
        
        unsafe {
            let mut delay: LARGE_INTEGER = core::mem::zeroed();
            *delay.QuadPart_mut() = -2 * 10_000_000;
            NtDelayExecution(1, &mut delay);

            let mut count = 0;
            let status = NtAlertResumeThread(main_thread.0, &mut count);
        }
    }).unwrap();

    unsafe {
        let mut delay: LARGE_INTEGER = core::mem::zeroed();
        *delay.QuadPart_mut() = i64::MIN;
        NtDelayExecution(1, &mut delay);
    }

    handle.join().unwrap();

    0
}