

use std::{collections::{HashMap, HashSet}, fs::{self, File}, path::PathBuf};
use capstone::{arch::{self, BuildsCapstone, BuildsCapstoneSyntax}, Capstone};
use log::*;
use walkdir::WalkDir;
use crate::{process_dumper::ProcessDumper, processor::ProcessorStep, types::DisassemblerConfig, utils::save_pretty_hex_dump_from_slice};
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
       
        let mut cs = Capstone::new()
            .x86()
            .mode(arch::x86::ArchMode::Mode64)
            .syntax(arch::x86::ArchSyntax::Intel)
            .build()?;

        cs.set_skipdata(true)?;
        cs.set_detail(true)?;

        let file_stem = self.exe_path.file_stem().unwrap().to_string_lossy().to_string();
        let file_name = self.exe_path.file_name().unwrap().to_string_lossy().to_string();
        let output_path = self.dest_path.join(file_stem);
        fs::create_dir_all(&output_path)?;
        let mut output_bin_path = ProcessDumper::get_bin_path(&self.exe_path, &self.dest_path);
        let mut process_dumper = ProcessDumper::new(&self.exe_path, &output_bin_path)?;
        let dump = process_dumper.get_cached()?;
        let mut filter = HashSet::new();
        filter.insert(file_name);


        for block in dump.blocks {

            let module_name = match &block.block.module_name {
                Some(name) if filter.contains(name) => name,
                Some(_) => continue,
                None => continue
            };

            if block.block.is_executable {

            }

            if block.block.is_readable {
                let file_name = format!("{}.txt", block.block.base);
                let file_path = output_path.join(file_name);

                let data = process_dumper.read_block_data(&block)?;
                // let file = File::create(file_path)?;
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