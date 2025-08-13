use std::{cell::RefCell, fs::File, io::Read, path::PathBuf, rc::Rc};

use flexi_logger::Logger;
use anyhow::Result;
use log::*;
use crate::{bus::{Bus, SharedBus}, cpu::Cpu, decoder::Decoder, emulator::Emulator, memory_region::MemoryRegion};

mod cpu;
mod registers;
mod memory_region;
mod emulator;
mod decoder;
mod bus;
mod types;
mod flags;
mod utils;
mod alu;

fn get_memory_region(file_path: &str) -> Result<MemoryRegion> {
    let (start_addr, size) = {
        let path = PathBuf::from(file_path);
        let stem = path.file_stem().unwrap().to_string_lossy();
        let mut parts = stem.split('_');
        let addr_str = parts.next().unwrap();
        let size_str = parts.next().unwrap();
        
        let addr_val = u64::from_str_radix(addr_str.trim_start_matches("0x"), 16).unwrap();
        let size_val = size_str.parse::<usize>().unwrap();

        (addr_val, size_val)
    };

    let mut file = File::open(file_path)?;
    let mut data = vec![];
    file.read_to_end(&mut data)?;

    let mut region = MemoryRegion::new(start_addr, size);
    region.write_bytes(start_addr, &data)?;

    Ok(region)
}

fn main() -> Result<()> {
    Logger::try_with_str("info")?.start()?;

    let bus: SharedBus = Rc::new(RefCell::new(Bus::new()));

    let file_path = r"C:\repos\rust_playground\app\target\debug\output\LOSTARK\PE\0x147E25000_4096_bpcbpmed.section";
    let code_region = get_memory_region(file_path)?;
    bus.borrow_mut().add_region(code_region);

    let file_path = r"C:\repos\rust_playground\app\target\debug\output\LOSTARK\PE\0x1475C3000_8790016_nuztkydr.section";
    let code_region = get_memory_region(file_path)?;
    bus.borrow_mut().add_region(code_region);

    let file_path = r"C:\repos\rust_playground\app\target\debug\output\LOSTARK\PE\0x140000000_4096_dos.data";
    let code_region = get_memory_region(file_path)?;
    bus.borrow_mut().add_region(code_region);

    let file_path = r"C:\repos\rust_playground\app\target\debug\output\LOSTARK\PE\0x1469EA000_12423168_2020202020202020.section";
    let code_region = get_memory_region(file_path)?;
    bus.borrow_mut().add_region(code_region);

    let rip = 0x147E25000;
    let stack_size = 64 * 1024usize;
    let stack_base = 0x7fff_ffff_0000 as u64;
    let rsp = stack_base + stack_size as u64;
    bus.borrow_mut().add_region(MemoryRegion::new(stack_base, stack_size));

    let mut cpu = Cpu::new(rip, rsp, bus.clone());
    
    let decoder = Decoder::new(bus)?;
    let mut emulator = Emulator::new(cpu, decoder);

    match emulator.run() {
        Ok(_) => {
            info!("Completed");
        },
        Err(err) => {
            error!("{}", err);
            emulator.dump()?;
        },
    }

    Ok(())
}
