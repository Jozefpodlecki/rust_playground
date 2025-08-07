use log::*;
use std::{collections::HashSet, fs::{self, File}, path::PathBuf};
use crate::{disassembler::Disassembler, process_dumper::ProcessDumper, processor::ProcessorStep, types::DisassemblerConfig, utils::save_pretty_hex_dump_from_slice};
use anyhow::*;


pub struct DisassembleProcessStep {
    config: DisassemblerConfig,
    exe_path: PathBuf,
    dest_path: PathBuf
}

impl ProcessorStep for DisassembleProcessStep {
    fn name(&self) -> String {
        format!("Disassemble process {:?}", self.exe_path.file_name().unwrap())
    }

    fn can_execute(&self) -> bool {
        true
    }

    fn execute(self: Box<Self>) -> Result<()> {

        let file_stem = self.exe_path.file_stem().unwrap().to_string_lossy().to_string();
        let file_name = self.exe_path.file_name().unwrap().to_string_lossy().to_string();
        let output_path = self.dest_path.join(file_stem);
        fs::create_dir_all(&output_path)?;
        let output_bin_path = ProcessDumper::get_bin_path(&self.exe_path, &self.dest_path);
        let mut process_dumper = ProcessDumper::new(&self.exe_path, &output_bin_path)?;
        let dump = process_dumper.get_cached()?;
        let disassembler = Disassembler::new()?;

        for block in dump.blocks {

            let module_name = match &block.block.module_name {
                Some(name) if self.config.filter.contains(name) => name,
                Some(_) => continue,
                None => continue
            };

            if block.block.is_executable {
               
                let file_name = format!("{}_{}.csv", module_name, block.block.base);
                let file_path = output_path.join(file_name);
                let data = process_dumper.read_block_data(&block)?;
                let mut cursor = 0;

                disassembler.export_to_csv(block.block.base, &data, &file_path)?;
            }

            if block.block.is_readable {
                let file_name = format!("{}_{}.txt", module_name, block.block.base);
                let file_path = output_path.join(file_name);
                let data = process_dumper.read_block_data(&block)?;
                save_pretty_hex_dump_from_slice(data, file_path, 32)?;
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
        Self {
            config,
            exe_path,
            dest_path
        }
    }
}