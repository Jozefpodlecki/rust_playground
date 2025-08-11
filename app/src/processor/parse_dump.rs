use log::*;
use std::{collections::{HashMap, HashSet}, fs::{self, File}, io::{BufWriter, Write}, path::PathBuf};
use anyhow::*;

use crate::{process::ProcessDump, processor::ProcessorStep};


pub struct ParseDumpStep {
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

        let dump = ProcessDump::open(self.dump_path)?;
        let mut map = HashMap::new();

        for (name, exports) in dump.exports {

            let module = dump.modules.get(&name).unwrap();

            if !module.is_dll {
                continue;
            }

            for export in exports {
                let name = export.name;
                let address = module.base + export.address;
                let value = format!("{}.{}", module.file_name, name);
                map.insert(address, value);
            }
        }

        Ok(())
    }
}

impl ParseDumpStep {
    pub fn new(
        exe_path: PathBuf,
        dest_path: PathBuf) -> Self {
        let file_stem = exe_path.file_stem().unwrap().to_string_lossy().to_string();
        let output_path = dest_path.join(file_stem);
        let dump_path = ProcessDump::get_path(&exe_path, &output_path);

        Self {
            dump_path,
            dest_path: output_path
        }
    }
}