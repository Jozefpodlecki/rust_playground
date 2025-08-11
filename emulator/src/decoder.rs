use std::{fs::File, io::BufReader, path::Path};

use anyhow::Result;
use decompiler_lib::decompiler::{stream::DisasmStream, types::Instruction, Disassembler};

pub struct Decoder {
    stream: DisasmStream<BufReader<File>>,
}

impl Decoder {
    pub fn new(file_path: &Path) -> Result<Self> {
        let file = File::open(file_path)?;
        let disassembler = Disassembler::from_file(file, 0x147E25000, 1000)?;
        let stream = disassembler.disasm_all()?;

        Ok(Self { stream })
    }

    pub fn decode_next(&mut self) -> Option<Instruction> {
       
        self.stream.next()
    }

}