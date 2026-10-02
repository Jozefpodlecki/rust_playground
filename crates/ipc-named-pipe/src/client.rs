use core::time::Duration;

use log::*;
use win_platform::{rng::Rng, utils::Sleeper};

use crate::{
    api::AsyncIpcClient, error::IpcError, random, types::{DebugEvent, DebugEventKind, DebugVerdict, DebugVerdictKind},
};

pub extern "system" fn on_client() {
    info!("Client: Spawned");
    Sleeper::sleep(Duration::from_secs(1));

    match run() {
        Ok(()) => error!("Client: Early return?"),
        Err(err) => error!("Client: {err}"),
    }
}

fn run() -> Result<(), IpcError> {
    info!("Client: Connecting");
    let client = AsyncIpcClient::open()?;
    let mut rng = Rng::from_shared_data();
    info!("Client: Connected");

    loop {
        let event = client.read()?;
        info!("Client: received {event:?}");
        let verdict = random::verdict(event, &mut rng);
        client.write(&verdict)?;
    }
}