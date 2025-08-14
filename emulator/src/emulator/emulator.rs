use anyhow::Result;
use log::info;
use crate::{cpu::Cpu, decoder::Decoder, emulator::snapshot::{Snapshot, SnapshotStore}};

pub struct Emulator<'a> {
    cpu: Cpu<'a>,
    decoder: Decoder<'a>,
    snapshot_store: &'a SnapshotStore,
    tick_count: u64
}

impl<'a> Emulator<'a> {
    pub fn new(
        cpu: Cpu<'a>,
        decoder: Decoder<'a>,
        snapshot_store: &'a SnapshotStore) -> Self {
        Self {
            cpu,
            decoder,
            snapshot_store,
            tick_count: 0
        }
    }

    pub fn run(&mut self) -> Result<()> {
        
        loop {
          
            let instruction = self.decoder.decode_next(self.cpu.rip)?;

            info!("{}", instruction);

            self.cpu.handle(instruction)?;
            self.tick_count += 1;

            if self.tick_count % 200000 == 0 {
                let snapshot = self.snapshot()?;
                info!("Saving snapshot");
                self.snapshot_store.save(&snapshot)?;
            }
        }
    }

    pub fn snapshot(&self) -> Result<Snapshot> {
        let mut snapshot = Snapshot::default();
        let bus = &self.cpu.bus;
        let regions = bus.get_regions();

        for region in regions {
            snapshot.regions.push(region.clone());
        }

        snapshot.rip = self.cpu.rip;
        snapshot.rflags = self.cpu.rflags.raw();
        snapshot.registers = self.cpu.registers.clone();

        Ok(snapshot)
    }
}