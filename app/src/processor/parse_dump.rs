use log::*;
use object::Object;
use serde::Serialize;
use std::{collections::{BTreeMap, HashMap, HashSet}, fs::{self, File}, io::{BufWriter, Write}, path::PathBuf};
use anyhow::*;

use crate::{config::DisassemblerConfig, disassembler::{utils::DisassemblerExtensions, Disassembler}, misc::export_dump::ExportDump, process::{ProcessDump, ProcessModule, ProcessModuleExport}, processor::ProcessorStep};


pub struct ParseDumpStep {
    config: DisassemblerConfig,
    dump_path: PathBuf,
    dest_path: PathBuf
}

impl ProcessorStep for ParseDumpStep {
    fn name(&self) -> String {
        format!("Parse dump {:?}", self.dump_path.file_name().unwrap())
    }

    fn can_execute(&self) -> bool {
        
        if !self.dump_path.exists() {
            return false
        }
        
        true
    }

    fn execute(self: Box<Self>) -> Result<()> {

        let file_name = self.dump_path.file_name().unwrap().to_string_lossy().to_string();
        let dump = ProcessDump::open(self.dump_path)?;
       
        let exports_map = get_module_exports_map(&dump.modules, &dump.exports);

        for block in dump.blocks {

            let address = block.base;
            let buf_size = 10000;
            let module_name = match &block.module_name {
                Some(name) if self.config.filter.contains(name) => name,
                Some(_) => continue,
                None => continue
            };

            if block.is_executable {
                let reader = block.read_data()?;

                let disassembler = Disassembler::from_reader(reader, address, buf_size)?;
                let stream = disassembler.disasm_all()?;
                let file_name = format!("{}_{}_calls.json", module_name, block.base);
                let file_path = self.dest_path.join(&file_name);

                info!("Extracting calls from region 0x{:X}", address);
                let calls_map = stream.get_calls()?;

                let writer = File::create(file_path)?;
                serde_json::to_writer_pretty(writer, &calls_map)?;

                for (key, value) in calls_map {
                    if let Some(entry) = exports_map.get(&value) {
                        info!("Found {} at {}", entry, key);
                    }
                }
            }
        }

        Ok(())
    }
}

impl ParseDumpStep {
    pub fn new(
        config: DisassemblerConfig,
        exe_path: PathBuf,
        dest_path: PathBuf) -> Self {
        let file_stem = exe_path.file_stem().unwrap().to_string_lossy().to_string();
        let output_path = dest_path.join(file_stem);
        let dump_path = ProcessDump::get_path(&exe_path, &dest_path);

        Self {
            config,
            dump_path,
            dest_path: output_path
        }
    }
}


fn get_module_exports_map(
    modules: &HashMap<String, ProcessModule>,
    exports: &HashMap<String, Vec<ProcessModuleExport>>
) -> BTreeMap<String, String> {
    let mut map: BTreeMap<String, String> = BTreeMap::new();

    for (name, exports) in exports {

        let module = modules.get(name).unwrap();

        if !module.is_dll {
            continue;
        }

        for export in exports {
            let name = export.name.clone();
            let address = module.base + export.address;
            let key = format!("0x{:X}", address);
            let value = format!("{}.{}", module.file_name, name);
            map.insert(key, value);
        }
    }

    map
}