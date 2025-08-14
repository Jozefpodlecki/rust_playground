use std::{collections::HashMap, fs::File, path::PathBuf, thread::sleep, time::Duration};

use flexi_logger::Logger;
use anyhow::Result;
use log::*;
use emulator::*;
use serde_json::Value;

use crate::emulator::snapshot::{Snapshot, SnapshotStore};

mod emulator;

fn main() -> Result<()> {
    Logger::try_with_str("debug")?
        // .log_to_file(FileSpec::default())
        // .write_mode(WriteMode::BufferAndFlush)
        .start()?;

    let snapshot_store = SnapshotStore::new()?;

    let snapshot = snapshot_store.latest().or_else(|| {
        info!("Didn't find latest snapshot");
        create_snapshot().ok()
    }).unwrap();
    
    let Snapshot {
        regions,
        registers,
        rflags,
        rip
    } = snapshot;

    info!("RIP: 0x{:X} RSP: 0x{:X}", rip, registers.rsp);

    for region in regions.iter() {
        info!("{}", region);
    }

    sleep(Duration::from_secs(5));

    let mut bus = Bus::from_regions(regions);
    let bus_ptr = &mut bus as *mut Bus;
    let bus_mut: &mut Bus = unsafe { &mut *bus_ptr };
    let decoder = Decoder::new(&bus)?;
    let cpu = Cpu::new(rip, registers, rflags.into(), bus_mut);
    let mut emulator = Emulator::new(cpu, decoder, &snapshot_store);

    match emulator.run() {
        Ok(_) => {
            info!("Completed");
        },
        Err(err) => {
            error!("{}", err);
            let snapshot = emulator.snapshot()?;
            snapshot_store.save(&snapshot)?;
        },
    }

    Ok(())
}
