use core::{ptr::null_mut, time::Duration};

use alloc::boxed::Box;
use log::{error, info};
use ntapi::ntpsapi::NtCurrentThreadId;
use win_platform::{
    KUserSharedData, NtError,
    rng::Rng,
    syscalls::NtCreateThreadEx,
    types::{HANDLE, NtCurrentProcess},
    utils::Sleeper,
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
    let mut rng = Rng::from_shared_data();

    loop {
        let timeout = rng.random_range(5..10);
        create_thread(send_event as _);
        info!("Thread spawner: next thread in {timeout} seconds");
        Sleeper::sleep(Duration::from_secs(timeout));
    }
}

pub extern "system" fn send_event() {
    Sleeper::sleep(Duration::from_secs(2));
    let tid = unsafe { NtCurrentThreadId() as u32 };
    let mut rng = Rng::from_shared_data();
    let iterations = rng.random_range(5..10);

    info!("Server thread: Initialized {iterations} iterations");

    let cycle = move || {

        let server = SERVER.get();

        let event = DebugEvent { tid, kind: DebugEventKind::ThreadCreated };
        register_or_reset_thread(tid);
        server.write(&event)?;
        wait_for_verdict(tid);

        for _ in 1..iterations {
            let event = DebugEvent { tid, kind: DebugEventKind::Breakpoint };
            register_or_reset_thread(tid);
            server.write(&event)?;
            wait_for_verdict(tid);
        }

        let event = DebugEvent { tid, kind: DebugEventKind::ThreadExited };
        register_or_reset_thread(tid);
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

fn register_or_reset_thread(tid: u32) {
    let mut guard = REGISTRY.0.write();
    let entry = guard.entry(tid);

    match entry {
        heapless::linear_map::Entry::Occupied(mut e) => {
            let thread = e.get_mut();
            thread.timestamp();
            thread.verdict = DebugVerdict { tid, kind: DebugVerdictKind::None };
        }
        heapless::linear_map::Entry::Vacant(e) => {
            let thread = DebugThread {
                updated_on: KUserSharedData::system_time(),
                verdict: DebugVerdict { tid, kind: DebugVerdictKind::None },
            };
            e.insert(Box::new(thread));
        }
    }
}