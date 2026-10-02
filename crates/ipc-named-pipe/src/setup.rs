use core::{cell::SyncUnsafeCell, ptr::null_mut, time::Duration};

use alloc::boxed::Box;
use heapless::LinearMap;
use log::*;
use logger::ConsoleLoggerBuilder;
use ntapi::ntpsapi::{NtCurrentThreadId, NtWaitForAlertByThreadId};
use win_platform::{KUserSharedData, NtError, syscalls::NtCreateThreadEx, types::{HANDLE, NtCurrentProcess, SystemTime}, utils::Sleeper};
use winapi::um::winnt::THREAD_ALL_ACCESS;

use crate::{client::IpcClient, error::IpcError, server::IpcServer, types::*};

static mut SERVER: SyncUnsafeCell<Option<IpcServer>> = SyncUnsafeCell::new(None);
static mut REGISTRY: ThreadRegistry = ThreadRegistry::new();

const MAX_THREADS: usize = 40;

pub struct ThreadRegistry(spin::RwLock<LinearMap<u32, Box<DebugThread>, MAX_THREADS>>);

impl ThreadRegistry {
    pub const fn new() -> Self {
        Self(spin::RwLock::new(LinearMap::new()))
    }
}

pub struct DebugThread {
    updated_on: SystemTime,
    verdict: DebugVerdict
}

pub fn setup() -> Result<(), IpcError> {

    ConsoleLoggerBuilder::new()
        .map_err(IpcError::Logger)?
        .build()
        .map_err(IpcError::Logger)?;

    info!("ConsoleLogger = {}", core::mem::size_of::<logger::ConsoleLogger>());
    let server = IpcServer::create()?;
    info!("Created pipe");

    for _ in 1..4 {
        create_thread(send_event as _);
    }
        
    create_thread(on_client as _);

    unsafe {
        *SERVER.get() = Some(server);
        let guard = SERVER.get_mut();
        guard.as_ref().unwrap_unchecked().listen()?;
    };

    info!("Found client?");

    loop {
        // let event = unsafe {
        //     let guard = SERVER.get_mut();
        //     let server = guard.as_ref().unwrap_unchecked();
        //     server.read()
        // };
        
        // match event {
        //     Ok(event) => {
        //         info!("{event:?}");
        //     },
        //     Err(err) => {
        //         info!("Dispatcher {err}");
        //         Sleeper::sleep(Duration::from_secs(1));
        //     },
        // }

        Sleeper::sleep(Duration::from_secs(60));
    }

    Ok(())
}

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
        null_mut()).ok()?;

    Ok(handle)
}

pub extern "system" fn on_client() {
    Sleeper::sleep(Duration::from_secs(1));
    
    let looper = move || {
        let client = IpcClient::open()?;
        info!("Client: Initialized?");

        loop {
            match client.read() {
                Ok(event) => {
                    info!("{event:?}");
                },
                Err(err) => {
                    error!("{err}");
                },
            }
        }

        Ok::<(), IpcError>(())
    };

    match looper() {
        Ok(_) => {
            error!("Client: Early return?")
        },
        Err(err) => {
            error!("Client: {err}")
        },
    }
}

pub extern "system" fn send_event() {
    Sleeper::sleep(Duration::from_secs(2));
    let tid = unsafe { NtCurrentThreadId() as u32 };
    info!("Initialized");

    loop {
        unsafe {
            let event = DebugEvent {
                tid,
                kind: DebugEventKind::Breakpoint,
            };

            let mut guard = REGISTRY.0.write();
            let entry = guard.entry(tid);

            match entry {
                heapless::linear_map::Entry::Occupied(mut entry) => {
                    let entry = entry.get_mut();
                    entry.updated_on = KUserSharedData::system_time();
                    entry.verdict = DebugVerdict {
                        tid,
                        kind: DebugVerdictKind::None
                    };
                },
                heapless::linear_map::Entry::Vacant(entry) => {
                    let thread = DebugThread {
                        updated_on: KUserSharedData::system_time(),
                        verdict: DebugVerdict {
                            tid,
                            kind: DebugVerdictKind::None
                        }
                    };
                    entry.insert(Box::new(thread));
                },
            }

            let guard = SERVER.get_mut();
            let server = guard.as_ref().unwrap_unchecked();
            info!("Sending event");
            match server.write(&event) {
                Ok(_) => {
                    info!("Sent event");
                },
                Err(err) => {
                    error!("Server: {err}");
                },
            }

            info!("Waiting for verdict");
            let guard = REGISTRY.0.read();
            let entry = guard.get(&tid).unwrap_unchecked();
            let addr = &entry as *const _ as _;
            let status = NtWaitForAlertByThreadId(addr, null_mut());
            info!("Received verdict");

            Sleeper::sleep(Duration::from_secs(1));
        }
    }
}