use std::{fs::{self, File}, path::Path};

use anyhow::*;
use bincode::{config::Configuration, Decode, Encode};
use once_cell::sync::Lazy;
use windows::Win32::Foundation::HANDLE;

use crate::process_dumper::{memory::{dump_memory_regions, enumerate_modules}, types::{MemoryBlock, ProcessModule}, utils::{get_windows_version, match_module}};

static BINCODE_CONFIG: Lazy<Configuration> = Lazy::new(|| bincode::config::standard());

#[derive(Debug, Decode, Encode, Clone)]
pub struct ProcessDumpResult {
    pub win_version: String,
    pub modules: Vec<ProcessModule>,
    pub blocks: Vec<MemoryBlock>,
}

impl ProcessDumpResult {
    pub fn new(process_handle: HANDLE) -> Result<Self> {

        unsafe {
            let win_version = get_windows_version()?;
            let modules = enumerate_modules(process_handle)?;
            let mut blocks = dump_memory_regions(process_handle)?;

            for block in blocks.iter_mut() {
                block.module = match_module(block.base, &modules).cloned();
            }

            Ok(Self {
                win_version,
                blocks,
                modules
            })
        }        
    }

    pub fn encode_to_file(&self, path: &Path) -> Result<()> {
        let mut file = File::create(path)?;
        let encoded = bincode::encode_into_std_write(self, &mut file, *BINCODE_CONFIG)?;
        Ok(())
    }

    pub fn decode_from_file(path: &Path) -> Result<Self> {
        let mut file = File::open(path)?;
        let decoded = bincode::decode_from_std_read(&mut file, *BINCODE_CONFIG)?;
        Ok(decoded)
    }
}

/*
let process_dumper = ProcessDumper::new();

let mut data = process_dumper.run_or_get_cached(&args)?;
data.blocks.sort_by_key(|pr| pr.size);

for data in data.blocks.iter() {

    if !(data.is_executable
        || data.is_readable)
        || (data.module.as_ref().filter(|pr| pr.file_name == "LOSTARK.exe").is_none()
        && data.module.as_ref().filter(|pr| pr.file_name == "EFEngine.dll").is_none()) {
        continue;
    }

    info!("file_name:{:?} size:{} is_executable:{} is_readable: {}",
        data.module.as_ref().map(|pr| &pr.file_name),
        data.size,
        data.is_executable,
        data.is_readable);
    }
*/