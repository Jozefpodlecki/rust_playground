use std::{collections::HashMap, io::{Read, Write}};

use anyhow::*;
use byteorder::{LittleEndian, ReadBytesExt};
use log::debug;
use object::Object;
use windows::Win32::System::SystemInformation::{GetVersionExW, OSVERSIONINFOW};

use crate::process::types::{ProcessModule, ProcessModuleExport};

pub unsafe fn get_windows_version() -> Result<String> {
    let mut info = OSVERSIONINFOW::default();
    info.dwOSVersionInfoSize = std::mem::size_of::<OSVERSIONINFOW>() as u32;

    GetVersionExW(&mut info as *mut _ as *mut _)?;

    Ok(format!("{}.{}.{}", info.dwMajorVersion, info.dwMinorVersion, info.dwBuildNumber))
}

pub fn match_module<'a>(
    base: u64,
    modules: &'a [ProcessModule],
) -> Option<&'a ProcessModule> {
    modules.iter().find(|module| {
        let module_start = module.base as usize;
        let module_end = module_start + module.size as usize;
        let block_base = base as usize;

        block_base >= module_start && block_base < module_end
    })
}

pub fn write_string<W: Write>(writer: &mut W, str: &str) -> Result<()> {
    let bytes = str.as_bytes();
    writer.write_all(&(bytes.len() as u32).to_le_bytes())?;
    writer.write_all(bytes)?;
    Ok(())
}

pub fn read_string<R: Read>(reader: &mut R) -> Result<String> {
    let len = reader.read_u32::<LittleEndian>()? as usize;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;

    String::from_utf8(buf).map_err(|e| anyhow!("Invalid UTF-8 string: {}", e))
}

pub fn read_bool<R: Read>(reader: &mut R) -> Result<bool> {
    let byte = reader.read_u8()?;
    match byte {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(anyhow!("Invalid boolean value: {}", byte)),
    }
}

pub fn collect_exports(modules: &[ProcessModule]) -> Result<HashMap<String, Vec<ProcessModuleExport>>> {
    let mut exports = HashMap::new();

    for module in modules {
        if !module.is_dll {
            continue;
        }

        let data = std::fs::read(&module.file_path)?;
        let obj_file = object::File::parse(&*data)?;
        let module_exports: &mut Vec<ProcessModuleExport> = exports.entry(module.file_name.clone()).or_default();
        
        debug!("Finding module exports for {}", module.file_path.to_str().unwrap());
        for export in obj_file.exports()? {
            let name_bytes = export.name();
            let address = export.address();
            let name = match std::str::from_utf8(name_bytes) {
                std::result::Result::Ok(s) => s.to_string(),
                Err(_) => format!("sub_{}_{:X}", hex::encode(name_bytes), address),
            };

            debug!("Module export {}:address", name);
            module_exports.push(ProcessModuleExport { name, address });
        }
    }

    Ok(exports)
}