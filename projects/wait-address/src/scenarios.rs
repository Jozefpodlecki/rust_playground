use core::{panic::PanicInfo, sync::atomic::{AtomicU32, AtomicUsize, Ordering}};

use ntapi::{ntexapi::NtCreateEvent, ntobapi::NtWaitForSingleObject, ntpsapi::{NtAlertThreadByThreadId, NtCurrentProcess, NtCurrentThread, NtQueueApcThreadEx, NtWaitForAlertByThreadId}, winapi_local::um::winnt::NtCurrentTeb};
use toolkit::{Sleeper, Thread, ThreadHandle, ThreadOpenFlags, futex::{wait_on_address, wake_by_address_single}, println};
use winapi::{shared::{ntdef::{HANDLE, NotificationEvent, PLARGE_INTEGER, PVOID}, ntstatus::{STATUS_ALERTED, STATUS_SUCCESS, STATUS_USER_APC}}, um::winnt::{EVENT_ALL_ACCESS, LARGE_INTEGER}};

pub fn scenario_wait_on_address() {
    static VALUE: AtomicU32 = AtomicU32::new(0);
    
    let waiter = Thread::spawn(|| {
        println!("[Waiter] Waiting for VALUE to become non-zero...");
        
        loop {
            let current = VALUE.load(Ordering::Acquire);
            if current != 0 {
                println!("[Waiter] Woken up! VALUE = {}", current);
                break;
            }
            
            wait_on_address(&VALUE, 0u32, None);
        }
        
        println!("[Waiter] Exiting");
    }).unwrap();
    
    let waker = Thread::spawn(|| {
        Sleeper::sleep(1000);
        println!("[Waker] Setting VALUE to 42");
        VALUE.store(42, Ordering::Release);
        wake_by_address_single(&VALUE);
        println!("[Waker] Woke up waiter");
    }).unwrap();
    
    waiter.join().unwrap();
    waker.join().unwrap();
}

pub fn scenario_alert_by_thread_id() {
    static WAIT_ADDRESS: AtomicUsize = AtomicUsize::new(0);
    static THREAD_ID: AtomicU32 = AtomicU32::new(0);

    let waiter = Thread::spawn(|| {
        let teb = unsafe { NtCurrentTeb() };
        let thread_id = unsafe { (*teb).ClientId.UniqueThread as u32 };
        THREAD_ID.store(thread_id as u32, Ordering::Release);

        println!("[Waiter] Thread ID: {}", thread_id as u32);
        println!("[Waiter] Waiting for alert...");

        let address = &WAIT_ADDRESS as *const AtomicUsize as PVOID;
        WAIT_ADDRESS.store(address as usize, Ordering::Release);
        let timeout: PLARGE_INTEGER = core::ptr::null_mut();

        let status = unsafe { NtWaitForAlertByThreadId(address, timeout) };

        if status == STATUS_SUCCESS || status == STATUS_ALERTED {
            println!("[Waiter] Alerted! (status: 0x{:X})", status);
        } else {
            println!("[Waiter] Wait failed: 0x{:X}", status);
        }
    }).unwrap();

    let waker = Thread::spawn(|| {
        Sleeper::sleep(2000);

        let tid = THREAD_ID.load(Ordering::Acquire);
        if tid != 0 {
            println!("[Waker] Alerting thread {}", tid);
            let status = unsafe { NtAlertThreadByThreadId(tid as HANDLE) };
            if status != STATUS_SUCCESS {
                println!("[Waker] Alert failed: 0x{:X}", status);
            }
        }
    }).unwrap();

    waiter.join().unwrap();
    waker.join().unwrap();
}

pub fn scenario_queue_apc_thread_ex() {
    static THREAD_ID: AtomicU32 = AtomicU32::new(0);
    static FLAG: AtomicU32 = AtomicU32::new(0);
    static HANDLE: AtomicUsize = AtomicUsize::new(0);

    let waiter = Thread::spawn(|| {
        let teb = unsafe { NtCurrentTeb() };
        let thread_id = unsafe { (*teb).ClientId.UniqueThread as u32 };
        THREAD_ID.store(thread_id, Ordering::Release);

        let thread_handle = ThreadHandle::open(thread_id as _, ThreadOpenFlags::ALL).unwrap();
        HANDLE.store(thread_handle.handle() as _, Ordering::Release);

        println!("[Waiter] Thread ID: {}, Handle: {:p}", thread_id, thread_handle.handle());
        let mut delay: LARGE_INTEGER = unsafe { core::mem::zeroed() };
        unsafe { *delay.QuadPart_mut() = -1_000_000_000; };

        // let mut event_handle: HANDLE = core::ptr::null_mut();
        // let status = unsafe {
        //     NtCreateEvent(
        //         &mut event_handle,
        //         EVENT_ALL_ACCESS,
        //         core::ptr::null_mut(),
        //         NotificationEvent,
        //         0, // Initially not signaled
        //     )
        // };

        // let status = unsafe { NtWaitForSingleObject(event_handle, 1, &mut delay) };
        let status = unsafe { NtWaitForSingleObject(NtCurrentProcess, 1, &mut delay) };
        
        if status == STATUS_USER_APC {
            println!("[Waiter] APC! (status: 0x{:X})", status);
        } else {
            println!("[Waiter] Wait failed: 0x{:X}", status);
        }

    }).unwrap();

    let waker = Thread::spawn(|| {
        Sleeper::sleep(1000);
        let teb = unsafe { NtCurrentTeb() };
        let thread_id = unsafe { (*teb).ClientId.UniqueThread as u32 };
        println!("[Waker] ThreadId: {}", thread_id);

        let handle = HANDLE.load(Ordering::Acquire) as HANDLE;
        if !handle.is_null() {
            unsafe extern "C" fn apc_routine(
                arg1: PVOID,
                arg2: PVOID,
                arg3: PVOID
            ) {
                println!("[APC] Executing! arg1: {:p}, arg2: {:p}, arg3: {:p}", arg1, arg2, arg3);
            }

            println!("[Waker] Queueing APC... Handle: {:p}", handle);
            let status = unsafe {
                NtQueueApcThreadEx(
                    handle,
                    core::ptr::null_mut(),
                    Some(apc_routine),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                )
            };

            if status == STATUS_SUCCESS {
                println!("[Waker] APC queued successfully");
                FLAG.store(1, Ordering::Release);
            } else {
                println!("[Waker] APC queue failed: 0x{:X}", status);
            }
        }
    }).unwrap();

    waiter.join().unwrap();
    waker.join().unwrap();
}