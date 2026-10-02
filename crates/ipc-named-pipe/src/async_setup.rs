use core::time::Duration;

use log::info;
use logger::ConsoleLoggerBuilder;
use win_platform::utils::Sleeper;

use crate::{
    api::AsyncIpcServer,
    error::IpcError,
    spawn::{create_thread, send_event, thread_spawner},
    state::SERVER,
    verdict::apply_verdict,
    client::on_client,
};

pub fn setup() -> Result<(), IpcError> {
    ConsoleLoggerBuilder::new()
        .map_err(IpcError::Logger)?
        .build()
        .map_err(IpcError::Logger)?;

    let server = AsyncIpcServer::create()?;
    info!("Created pipe");

    create_thread(thread_spawner as _).map_err(IpcError::CouldNotCreate)?;
    create_thread(on_client as _).map_err(IpcError::CouldNotCreate)?;

    SERVER.set(server);
    SERVER.get().listen()?;

    info!("Found client");

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