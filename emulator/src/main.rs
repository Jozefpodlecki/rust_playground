use std::{collections::HashMap, fs::{self, File}, path::PathBuf, time::SystemTime};

use flexi_logger::{FileSpec, Logger, WriteMode};
use anyhow::Result;
use log::*;
use emulator::*;
use serde_json::Value;

use crate::emulator::snapshot::Snapshot;

mod emulator;

fn create_stack() -> MemoryRegion {
    let stack_size = 64 * 1024usize;
    let stack_base = 0x7fff_ffff_0000 as u64;
    MemoryRegion::new(stack_base, stack_size)
}

fn get_last_snapshot() -> Option<Snapshot> {
    let dir = ".";

    let mut latest: Option<(SystemTime, PathBuf)> = None;

    for entry in fs::read_dir(dir).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();

        if path.extension().and_then(|ext| ext.to_str()) != Some("snapshot") {
            continue;
        }

        let metadata = entry.metadata().ok()?;
        let created = metadata.created().or_else(|_| metadata.modified()).ok()?;

        match &latest {
            Some((latest_time, _)) if created <= *latest_time => {}
            _ => latest = Some((created, path)),
        }
    }

    latest.map(|(_, path)| Snapshot::get(&path)).flatten()
}

fn create_snapshot() -> Result<Snapshot> {
    
    let mut regions = vec![];
    let base_path = PathBuf::from(r"C:\repos\rust_playground\app\target\debug\output\LOSTARK\PE\");

    let file_path = base_path.join(r"summary.json");
    let file = File::open(file_path)?;
    let map: HashMap<String, Value> = serde_json::from_reader(file)?;
    let value = map.get("entry_point_va").unwrap();
    let rip = u64::from_str_radix(value.as_str().unwrap().trim_start_matches("0x"), 16).unwrap();

    let file_path = base_path.join(r"0x147E25000_4096_bpcbpmed.section");
    let region = get_memory_region(&file_path)?;
    regions.push(region);

    let file_path = base_path.join(r"0x1475C3000_8790016_nuztkydr.section");
    let region = get_memory_region(&file_path)?;
    regions.push(region);

    let file_path = base_path.join(r"0x140000000_4096_dos.data");
    let region = get_memory_region(&file_path)?;
    regions.push(region);

    let file_path = base_path.join(r"0x1469EA000_12423168_2020202020202020.section");
    let region = get_memory_region(&file_path)?;
    regions.push(region);

    let region = create_stack();
    let rsp = region.end_addr;
    regions.push(region);

    Ok(Snapshot { 
        rip,
        rflags: 0,
        registers: Registers::new(rsp),
        regions
    })
}

fn main() -> Result<()> {
    Logger::try_with_str("debug")?
        // .log_to_file(FileSpec::default())
        // .write_mode(WriteMode::BufferAndFlush)
        .start()?;

    let snapshot = get_last_snapshot().or_else(|| {
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

    let mut bus = Bus::from_regions(regions);
    let bus_ptr = &mut bus as *mut Bus;
    let bus_mut: &mut Bus = unsafe { &mut *bus_ptr };
    let decoder = Decoder::new(&bus)?;
    let cpu = Cpu::new(rip, registers, rflags.into(), bus_mut);
    let mut emulator = Emulator::new(cpu, decoder);

    match emulator.run() {
        Ok(_) => {
            info!("Completed");
        },
        Err(err) => {
            error!("{}", err);
            let snapshot = emulator.snapshot()?;
            snapshot.save()?;
        },
    }

    Ok(())
}
