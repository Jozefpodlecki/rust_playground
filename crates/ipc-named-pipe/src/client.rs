use core::time::Duration;

use log::{error, info};
use win_platform::{rng::Rng, utils::Sleeper};

use crate::{
    api::AsyncIpcClient, error::IpcError, random, types::{DebugEvent, DebugEventKind, DebugVerdict, DebugVerdictKind},
};

pub extern "system" fn on_client() {
    Sleeper::sleep(Duration::from_secs(1));

    match run() {
        Ok(()) => error!("Client: Early return?"),
        Err(err) => error!("Client: {err}"),
    }
}

fn run() -> Result<(), IpcError> {
    let client = AsyncIpcClient::open()?;
    let mut rng = Rng::from_shared_data();
    info!("Client: Initialized?");

    loop {
        let event = client.read()?;
        info!("Client: received {event:?}");
        let verdict = random::verdict(event, &mut rng);
        client.write(&verdict)?;
    }
}