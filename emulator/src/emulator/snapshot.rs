use std::path::Path;
use std::{fs::File, io::BufWriter};
use anyhow::Result;
use bincode::{Decode, Encode};
use chrono::Local;
use crate::emulator::{MemoryRegion, Registers};

#[derive(Debug, Default, Encode, Decode, Clone)]
pub struct Snapshot {
    pub rip: u64,
    pub rflags: u64,
    pub registers: Registers,
    pub regions: Vec<MemoryRegion>
}

impl Snapshot {
    pub fn save(self) -> Result<()> {
        
        let timestamp = Local::now().format("%H_%M_%S").to_string();
        let filename = format!("snapshot_{}.snapshot", timestamp);
        let file = File::create(filename)?;
        let mut writer = BufWriter::new(file);

        let config = bincode::config::standard();
        bincode::encode_into_std_write(self, &mut writer, config)?;

        Ok(())
    }

    pub fn get(path: &Path) -> Option<Self> {
        if !path.exists() {
            return None
        }

        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return None,
        };

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