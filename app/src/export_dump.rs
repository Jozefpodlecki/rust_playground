use std::{collections::{BTreeMap, HashMap}, fs::File, path::Path};
use anyhow::Result;
use crate::process::{ProcessModule, ProcessModuleExport};


pub struct ExportDump {

}

impl ExportDump {
    pub fn create(
        path: &Path, 
        modules: HashMap<String, ProcessModule>,
        exports: HashMap<String, Vec<ProcessModuleExport>>) -> Result<BTreeMap<String, String>> {
        let mut map= BTreeMap::new();

        for (name, exports) in exports {

            let module = modules.get(&name).unwrap();

            if !module.is_dll {
                continue;
            }

            for export in exports {
                let name = export.name;
                let address = module.base + export.address;
                let key = format!("0x{:X}", address);
                let value = format!("{}.{}", module.file_name, name);
                map.insert(key, value);
            }
        }

        let writer = File::create(path)?;
        serde_json::to_writer_pretty(writer, &map)?;
        
        Ok(map)
    }

    pub fn get(path: &Path) -> Result<BTreeMap<String, String>> {
        let reader = File::open(path)?;
        let map = serde_json::from_reader(reader)?;
        Ok(map)
    }
}