#![allow(unsafe_op_in_unsafe_fn)]

use std::collections::HashMap;
use std::ffi::{c_void, OsString};
use std::fmt;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::os::windows::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;
use anyhow::*;
use byteorder::{LittleEndian, ReadBytesExt};
use log::info;

mod utils;
mod process;
mod memory;
mod types;

use object::Object;
use windows::Win32::Foundation::HANDLE;

use crate::process_dumper::memory::*;
use crate::process_dumper::process::*;
use crate::process_dumper::types::*;
use crate::process_dumper::utils::*;
use crate::types::{AppConfig, LaunchMethod};

pub struct ProcessDumper {
    exists: bool,
    file: File
}

impl ProcessDumper {
    pub fn new(exe_path: &Path, output_path: &Path) -> Result<Self> {

        let file_name = exe_path.file_stem().unwrap().to_string_lossy();
        let output_bin_path = format!("{file_name}.bin");
        let output_bin_path = output_path.join(output_bin_path);
        let file;
        let exists;

        if output_bin_path.exists() {
            exists = true;
            file = File::open(output_bin_path)?;
        }
        else {
            exists = false;
            file = File::create(output_bin_path)?;
        }
        
        Ok(Self{
            exists,
            file
        })
    }

    pub fn get_cached(&mut self) -> Result<ProcessDump> {
        let mut reader = BufReader::new(&mut self.file);

        let dump = ProcessDump::decode(&mut reader)?;

        Ok(dump)
    }

    pub fn run(&mut self, exe_args: &[String], strategy: LaunchMethod) -> Result<ProcessDump> {
       
        let result = unsafe { self.run_inner(exe_args, strategy)? };

        Ok(result)
    }

    unsafe fn run_inner(&mut self, exe_args: &[String], strategy: LaunchMethod) -> Result<ProcessDump> {

        let handle = spawn_process(exe_args)?;

        sleep(Duration::from_secs(1));

        let main_module = get_main_module(handle)?.unwrap();

        match strategy {
            LaunchMethod::Wait { wait: duration } => {
                info!("Sleeping for {} seconds", duration.as_secs());
                sleep(duration);
            },
            LaunchMethod::Monitor { monitor: addr_offset } => {
                info!("Monitoring address 0x{:X}", addr_offset);
                let address = main_module.base + addr_offset;
                let wait_interval = Duration::from_secs(1);
                monitor_address(handle, address, wait_interval)?;
            },
        }

        suspend_process(handle)?;

        let result = self.write_to_file(handle)?;

        resume_process(handle)?;
        terminate_process(handle)?;
        close_handle(handle)?;

        Ok(result)
    }

    pub fn read_block_data(&mut self, block: &SerializedMemoryBlock) -> Result<Vec<u8>> {
        self.file.seek(SeekFrom::Start(block.data_offset))?;
        let mut data = vec![0u8; block.block.size as usize];
        self.file.read_exact(&mut data)?;
        Ok(data)
    }

    fn write_to_file(&mut self, handle: HANDLE) -> Result<ProcessDump> {

        let mut writer = BufWriter::new(&mut self.file);

        let win_version = unsafe { get_windows_version()? };
        let modules = unsafe { enumerate_modules(handle)? };
        let mut block_iter = MemoryRegionIterator::new(handle);
        let mut blocks = vec![];
        let mut exports = collect_exports(&modules)?;
        let module_map: HashMap<String, ProcessModule> = modules
            .clone()
            .into_iter()
            .map(|m| (m.file_name.clone(), m))
            .collect();

        write_string(&mut writer, &win_version)?;

        writer.write_all(&(module_map.len() as u32).to_le_bytes())?;
        for (name, module) in &module_map {
            write_string(&mut writer, &name)?;
            writer.write_all(&module.order.to_le_bytes())?;
            writer.write_all(&[module.is_dll as u8])?;
            write_string(&mut writer, &module.file_path.to_string_lossy())?;
            write_string(&mut writer, &module.file_name)?;
            writer.write_all(&module.entry_point.to_le_bytes())?;
            writer.write_all(&module.size.to_le_bytes())?;
            writer.write_all(&module.base.to_le_bytes())?;
        }

        writer.write_all(&(exports.len() as u32).to_le_bytes())?;
        for (module_name, exports) in &exports {
            write_string(&mut writer, module_name)?;

            writer.write_all(&(exports.len() as u32).to_le_bytes())?;
            for export in exports {
                write_string(&mut writer, &export.name)?;
                writer.write_all(&export.address.to_le_bytes())?;
            }
        }

        let mut count = 0u32;
        writer.write_all(&count.to_le_bytes())?;
       
        for block in block_iter {
            count += 1;
            
            let (mut block, data) = block?;

            block.module_name = match_module(block.base, &modules)
                .map(|pr| &pr.file_name).cloned();

            let mut serialized = SerializedMemoryBlock {
                data_offset: 0,
                block,
            };

            serialized.encode(&mut writer)?;
            serialized.data_offset = writer.stream_position()?;
            writer.write_all(&(data.len() as u32).to_le_bytes())?;
            writer.write_all(&data)?;

            blocks.push(serialized);
        }

        writer.seek(SeekFrom::Start(0))?;
        writer.write_all(&count.to_le_bytes())?;

        let dump = ProcessDump {
            win_version,
            exports,
            modules: module_map,
            blocks
        };

        Ok(dump)
    }

    fn read_memory_blocks(&mut self) -> Result<Vec<SerializedMemoryBlock>> {
        let mut reader = BufReader::new(&mut self.file);

        let count = reader.read_u32::<LittleEndian>()?;
        let mut blocks = Vec::with_capacity(count as usize);

        for _ in 0..count {
            let block = SerializedMemoryBlock::decode(&mut reader)?;
            blocks.push(block);
        }

        Ok(blocks)
    }
}

