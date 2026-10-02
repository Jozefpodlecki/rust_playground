use core::time::Duration;

use log::info;
use logger::ConsoleLoggerBuilder;
use ntapi::ntpsapi::NtCurrentThreadId;
use win_platform::utils::Sleeper;

use crate::{
    api::AsyncIpcServer, client::on_client, error::IpcError, spawn::{create_thread, send_event, thread_spawner}, state::SERVER, types::{DebugEvent, DebugEventKind}, verdict::{apply_verdict, wait_for_verdict},
};

pub fn setup() -> Result<(), IpcError> {
    ConsoleLoggerBuilder::new()
        .map_err(IpcError::Logger)?
        .build()
        .map_err(IpcError::Logger)?;

    let server = AsyncIpcServer::create()?;
    info!("Dispatcher: Created pipe");

    SERVER.set(server);
    
    info!("Dispatcher: Spawning client thread");
    let client_handle = create_thread(on_client as _).map_err(IpcError::CouldNotCreate)?;

    info!("Dispatcher: Listening");
    SERVER.get().listen()?;

    info!("Dispatcher: Found client");

    info!("Dispatcher: Senting initial event");
    let tid = unsafe { NtCurrentThreadId() as u32 };
    let event = DebugEvent {
        tid,
        kind: DebugEventKind::Initialized,
    };
    SERVER.get().write(&event);
    // wait_for_verdict(tid)?;
    let verdict = SERVER.get().read()?;

    info!("Dispatcher: Received {verdict:?}");

    create_thread(thread_spawner as _).map_err(IpcError::CouldNotCreate)?;

    loop {
        let verdict = SERVER.get().read();

        match verdict {
            Ok(verdict) => {
                info!("Dispatcher: received verdict: {verdict:?}");
                apply_verdict(&verdict)?;
            }
            Err(err) => {
                info!("Dispatcher {err}");
                Sleeper::sleep(Duration::from_secs(1));
            }
        }
    }
}