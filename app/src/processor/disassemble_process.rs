use log::*;
use std::{collections::HashSet, fs::{self, File}, io::{BufWriter, Write}, path::PathBuf};
use crate::{disassembler::Disassembler, process::ProcessDump, processor::ProcessorStep, types::DisassemblerConfig, utils::save_pretty_hex_dump_from_slice};
use anyhow::*;


pub struct DisassembleProcessStep {
    config: DisassemblerConfig,
    exe_path: PathBuf,
    dest_path: PathBuf
}

impl ProcessorStep for DisassembleProcessStep {
    fn name(&self) -> String {
        format!("Disassemble {:?}", self.exe_path.file_name().unwrap())
    }

    fn can_execute(&self) -> bool {
        
        if !self.exe_path.exists() {
            return false
        }

        if self.dest_path.exists() {
            return false
        }

        true
    }

    fn execute(self: Box<Self>) -> Result<()> {

    
        let file_name = self.exe_path.file_name().unwrap().to_string_lossy().to_string();
        
        fs::create_dir_all(&self.dest_path)?;
        let dump_path = ProcessDump::get_path(&self.exe_path, &self.dest_path);
        let mut dump = ProcessDump::open(dump_path)?;
        let disassembler = Disassembler::new()?;

        for mut block in dump.blocks {

            let module_name = match &block.block.module_name {
                Some(name) if self.config.filter.contains(name) => name,
                Some(_) => continue,
                None => continue
            };

            if block.block.is_executable {
               
               let file_name = format!("{}_{}_exec.data", module_name, block.block.base);
                // let file_name = format!("{}_{}_exec.txt", module_name, block.block.base);
                // let file_name = format!("{}_{}.csv", module_name, block.block.base);
                let file_path = self.dest_path.join(file_name);
                let mut reader = block.read_data()?;
                let mut cursor = 0;

                let mut writer = BufWriter::new(File::create(&file_path)?);
                std::io::copy(&mut reader, &mut writer)?;
                disassembler.export_to_txt(block.block.base, reader, &file_path)?;
                // disassembler.export_to_csv(block.block.base, &data, &file_path)?;
            }

            if block.block.is_readable {
                let file_name = format!("{}_{}_read.txt", module_name, block.block.base);
                let file_path = self.dest_path.join(file_name);
                let data = block.read_data()?;
                // save_pretty_hex_dump_from_slice(data, file_path, 32)?;
            }
        }

        Ok(())
    }
}

impl DisassembleProcessStep {
    pub fn new(
        config: DisassemblerConfig,
        exe_path: PathBuf,
        dest_path: PathBuf) -> Self {
        let file_stem = exe_path.file_stem().unwrap().to_string_lossy().to_string();
        let output_path = dest_path.join(file_stem);

        Self {
            config,
            exe_path,
            dest_path: output_path
        }
    }
}