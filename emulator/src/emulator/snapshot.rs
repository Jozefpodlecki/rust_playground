use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::{fs::File, io::BufWriter};
use anyhow::Result;
use bincode::{Decode, Encode};
use chrono::Local;
use log::info;
use crate::emulator::{MemoryRegion, Registers};

pub struct SnapshotStore {
    save_dir: PathBuf,
}

impl SnapshotStore {
    pub fn new() -> Result<Self> {
        let current_exe = std::env::current_exe()?;
        let save_dir = current_exe.parent().unwrap().to_path_buf().join("snapshots");
        fs::create_dir_all(&save_dir)?;
        Ok(Self { save_dir })
    }

    pub fn latest(&self) -> Option<Snapshot> {
        let mut latest: Option<(SystemTime, PathBuf)> = None;

        for entry in fs::read_dir(&self.save_dir).ok()? {
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

    pub fn save(&self, snapshot: &Snapshot) -> Result<PathBuf> {
        let timestamp = Local::now().format("%H_%M_%S").to_string();
        let file_name = format!("snapshot_{}.snapshot", timestamp);
        let file_path = self.save_dir.join(file_name);

        let file = File::create(&file_path)?;
        let mut writer = BufWriter::new(file);

        let config = bincode::config::standard();
        bincode::encode_into_std_write(snapshot, &mut writer, config)?;

        Ok(file_path)
    }
}

#[derive(Debug, Default, Encode, Decode, Clone)]
pub struct Snapshot {
    pub rip: u64,
    pub rflags: u64,
    pub registers: Registers,
    pub regions: Vec<MemoryRegion>
}

impl Snapshot {

    pub fn get(path: &Path) -> Option<Self> {
        if !path.exists() {
            return None
        }

        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return None,
        };

        info!("Using snapshot {:?}", path);
        let config = bincode::config::standard();
        bincode::decode_from_std_read(&mut file, config).ok()
    }
}

    // pub fn dump(self) -> Result<()> {

    //     // let file = File::create("dump.txt")?;
    //     // let mut writer = BufWriter::new(file);

    //     // let bus = self.cpu.bus.borrow();
    //     // // let region = bus.find_region(0x1475c3043)?;
    //     // // let disassembler = Disassembler::from_memory(&region.data, region.start_addr, 1000)?;

    //     // // let stream = disassembler.disasm_all()?;

    //     // // for instr in stream {
    //     // //     writeln!(writer, "{}", instr)?;
    //     // // }

    //     // let region = bus.find_region(0x1469ea014)?;
    //     // let disassembler = Disassembler::from_memory(&region.data, region.start_addr, 1000)?;
    //     // let stream = disassembler.disasm_all()?;

    //     // for instr in stream {
    //     //     writeln!(writer, "{}", instr)?;
    //     // }

    //     // writeln!(writer, "")?;
    //     // writeln!(writer, "Registers:")?;
    //     // writeln!(writer, "RAX: {:#X}", self.cpu.registers.rax)?;
    //     // writeln!(writer, "RBX: {:#X}", self.cpu.registers.rbx)?;
    //     // writeln!(writer, "RCX: {:#X}", self.cpu.registers.rcx)?;
    //     // writeln!(writer, "RDX: {:#X}", self.cpu.registers.rdx)?;
    //     // writeln!(writer, "RSP: {:#X}", self.cpu.registers.rsp)?;
    //     // writeln!(writer, "RBP: {:#X}", self.cpu.registers.rbp)?;
    //     // writeln!(writer, "RSI: {:#X}", self.cpu.registers.rsi)?;
    //     // writeln!(writer, "RDI: {:#X}", self.cpu.registers.rdi)?;
    //     // writeln!(writer, "R8 : {:#X}", self.cpu.registers.r8)?;
    //     // writeln!(writer, "R9 : {:#X}", self.cpu.registers.r9)?;
    //     // writeln!(writer, "R10: {:#X}", self.cpu.registers.r10)?;
    //     // writeln!(writer, "R11: {:#X}", self.cpu.registers.r11)?;
    //     // writeln!(writer, "R12: {:#X}", self.cpu.registers.r12)?;
    //     // writeln!(writer, "R13: {:#X}", self.cpu.registers.r13)?;
    //     // writeln!(writer, "R14: {:#X}", self.cpu.registers.r14)?;
    //     // writeln!(writer, "R15: {:#X}", self.cpu.registers.r15)?;
    //     // writeln!(writer, "RIP: {:#X}", self.cpu.rip)?;
    //     // writeln!(writer, "RFLAGS: {:#X}", self.cpu.rflags)?;

    //     Ok(())
    // }