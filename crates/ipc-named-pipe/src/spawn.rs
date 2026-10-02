use core::{ptr::null_mut, time::Duration};

use alloc::boxed::Box;
use log::{error, info};
use ntapi::ntpsapi::NtCurrentThreadId;
use win_platform::{
    KUserSharedData, NtError, rng::Rng, syscalls::{NtClose, NtCreateThreadEx, NtWaitForSingleObject}, types::{HANDLE, NtCurrentProcess}, utils::Sleeper,
};
use winapi::um::winnt::THREAD_ALL_ACCESS;

use crate::{
    error::IpcError, state::{DebugThread, REGISTRY, SERVER}, types::*, verdict::wait_for_verdict,
};

pub fn create_thread(entry: *const ()) -> Result<HANDLE, NtError> {
    let mut handle = null_mut();

    NtCreateThreadEx(
        &mut handle,
        THREAD_ALL_ACCESS,
        null_mut(),
        NtCurrentProcess,
        entry as _,
        null_mut(),
        0,
        0,
        0,
        0,
        null_mut(),
    )
    .ok()?;

    Ok(handle)
}

pub extern "system" fn thread_spawner() {
    let cycle = move || {
        let mut rng = Rng::from_shared_data();
        let mut handles = heapless::Vec::<HANDLE, 5>::new();

        loop {
            if handles.len() == handles.capacity() {
                let handle = handles.remove(0);
                info!("Thread spawner: too many threads, waiting for the earliest one to finish");
                NtWaitForSingleObject(handle, 1, null_mut()).ok()?;
                let _ = NtClose(handle);
            }

            let timeout = rng.random_range(5..10);
            let handle = create_thread(send_event as _)?;
            handles.push(handle);
            info!("Thread spawner: next thread in {timeout} seconds");
            Sleeper::sleep(Duration::from_secs(timeout));
        }
        
        Ok::<(), IpcError>(())
    };

    match cycle() {
        Ok(()) => {},
        Err(err) => {
            error!("{err}");
        },
    };
}

pub extern "system" fn send_event() {
    Sleeper::sleep(Duration::from_secs(2));
    let tid = unsafe { NtCurrentThreadId() as u32 };
    let mut rng = Rng::from_shared_data();
    let iterations = rng.random_range(5..10);

    info!("Server thread: Initialized {iterations} iterations");

    let mut cycle = move || {

        let server = SERVER.get();

        let event = DebugEvent { tid, kind: DebugEventKind::ThreadCreated };
        REGISTRY.reset(tid);
        server.write(&event)?;
        wait_for_verdict(tid);

        for _ in 1..iterations {
            let kind = match rng.random_range(0..100) {
                0..=2 => DebugEventKind::ProcessExited,
                _ => DebugEventKind::Breakpoint,
            };
            let event = DebugEvent { tid, kind };
            REGISTRY.reset(tid);
            server.write(&event)?;
            wait_for_verdict(tid);
        }

        let event = DebugEvent { tid, kind: DebugEventKind::ThreadExited };
        REGISTRY.reset(tid);
        server.write(&event)?;
        wait_for_verdict(tid);

        Ok::<(), IpcError>(())
    };

    match cycle() {
        Ok(()) => {},
        Err(err) => {
            error!("{err}");
        },
    };
}