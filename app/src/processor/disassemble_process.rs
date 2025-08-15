use log::*;
use std::{collections::HashSet, fs::{self, File}, io::{BufWriter, Write}, path::PathBuf};
use crate::{config::DisassemblerConfig, disassembler::{utils::DisassemblerExtensions, Disassembler}, process::ProcessDump, processor::ProcessorStep, utils::*};
use anyhow::*;


pub struct DisassembleProcessStep {
    config: DisassemblerConfig,
    dump_path: PathBuf,
    dest_path: PathBuf
}

impl ProcessorStep for DisassembleProcessStep {
    fn name(&self) -> String {
        format!("Disassemble {:?}", self.dump_path.file_name().unwrap())
    }

    fn can_execute(&self) -> bool {
        
        self.dump_path.exists()
    }

    fn execute(self: Box<Self>) -> Result<()> {
        let file_name = self.dump_path.file_name().unwrap().to_string_lossy().to_string();

        fs::create_dir_all(&self.dest_path)?;
       
        let mut dump = ProcessDump::open(self.dump_path)?;
        let regions_path = self.dest_path.join("regions");
        fs::create_dir_all(&regions_path)?;

        for mut block in dump.blocks {

            let address = block.base;
            let module_name = match &block.module_name {
                Some(name) if self.config.filter.contains(name) => name,
                Some(_) => continue,
                None => continue
            };

            if block.is_executable {
               
                let file_name = format!("{}_{}_exec.txt", module_name, block.base);
                let file_path = regions_path.join(&file_name);

                if file_path.exists() {
                    info!("Skipping {:?}", file_path.strip_prefix(&self.dest_path));
                    continue;
                }

                info!("Creating {}", file_name);
                let reader = block.read_data()?;

                let buf_size = 1000;
                let disassembler = Disassembler::from_reader(reader, address, buf_size)?;
                let stream = disassembler.disasm_all()?;
                stream.export_to_txt(&file_path)?;
            }

            if block.is_readable {
                let file_name = format!("{}_{}_read.txt", module_name, address);
                let file_path = regions_path.join(&file_name);

                if file_path.exists() {
                    info!("Skipping {:?}", file_path.strip_prefix(&self.dest_path));
                    continue;
                }

                info!("Creating {}", file_name);
                let data = block.read_data()?;
                save_hex_dump_pretty_from_reader(data, file_path, 32)?;
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
        let dest_path = dest_path.join(file_stem);
        let dump_path = ProcessDump::get_path(&exe_path, &dest_path);

        Self {
            config,
            dump_path,
            dest_path,
        }
    }
}