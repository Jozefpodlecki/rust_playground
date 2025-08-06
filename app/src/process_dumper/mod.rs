#![allow(unsafe_op_in_unsafe_fn)]

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

use windows::Win32::Foundation::HANDLE;

use crate::process_dumper::memory::*;
use crate::process_dumper::process::*;
use crate::process_dumper::types::{MemoryBlock, ProcessModule, SerializedMemoryBlock};
use crate::process_dumper::utils::{get_windows_version, match_module};
use crate::types::{RunArgs, WaitStrategy};



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

    pub fn get_cached(&mut self) -> Result<Vec<SerializedMemoryBlock>> {
        let data = self.read_memory_blocks()?;
        Ok(data)
    }

    pub fn run_or_get_cached(
        &mut self,
        exe_args: &[String],
        strategy: WaitStrategy) -> Result<Vec<SerializedMemoryBlock>> {
       
        if self.exists {
            let data = self.read_memory_blocks()?;
            return Ok(data);
        };
        
        let result = unsafe { self.run_inner(exe_args, strategy)? };

        Ok(result)
    }

    unsafe fn run_inner(&mut self, exe_args: &[String], strategy: WaitStrategy) -> Result<Vec<SerializedMemoryBlock>> {

        let handle = spawn_process(exe_args)?;

        sleep(Duration::from_secs(1));

        let main_module = get_main_module(handle)?.unwrap();

        match strategy {
            WaitStrategy::None => {},
            WaitStrategy::Sleep(duration) => {
                info!("Sleeping for {} seconds", duration.as_secs());
                sleep(duration);
            },
            WaitStrategy::MonitorOffset(mut addr_offset, duration) => {
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

    fn write_to_file(&mut self, handle: HANDLE) -> Result<Vec<SerializedMemoryBlock>> {

        let mut count = 0u32;
        let mut writer = BufWriter::new(&mut self.file);
        writer.write_all(&count.to_le_bytes())?;

        let win_version = unsafe { get_windows_version()? };
        let modules = unsafe { enumerate_modules(handle)? };
        let mut block_iter = MemoryRegionIterator::new(handle);
        let mut blocks = vec![];

        for module in modules.iter() {
            if !module.is_dll {
                continue;
            }

            let data = fs::read(&module.file_path)?;
            let obj_file = object::File::parse(&*data)?;
        }
        

        for block in block_iter {
            count += 1;
            
            let (mut block, data) = block?;

            block.module = match_module(block.base, &modules).cloned();

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

        Ok(blocks)
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

