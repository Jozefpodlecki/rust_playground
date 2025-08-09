#![allow(unsafe_op_in_unsafe_fn)]

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;
use anyhow::*;
use byteorder::{LittleEndian, ReadBytesExt};
use log::*;
use windows::Win32::Foundation::HANDLE;

mod utils;
mod process;
mod memory;
mod types;
mod thread;

use crate::process_dumper::memory::*;
use crate::process_dumper::process::*;
use crate::process_dumper::thread::get_threads;
use crate::process_dumper::types::*;
use crate::process_dumper::utils::*;
use crate::types::LaunchMethod;

pub struct ProcessDumper {
    exists: bool,
    exe_path: PathBuf,
    file: File
}

impl ProcessDumper {    
    pub fn new(exe_path: &Path, output_bin_path: &Path) -> Result<Self> {

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
            exe_path: exe_path.to_path_buf(),
            file
        })
    }

    pub fn get_bin_path(exe_path: &Path, output_path: &Path) -> PathBuf {
        let file_name = exe_path.file_stem().unwrap().to_string_lossy();
        let output_bin_path = format!("{file_name}.bin");
        let output_bin_path = output_path.join(output_bin_path);

        output_bin_path
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

        let mut exe_args = exe_args.to_vec();
        exe_args.insert(0, self.exe_path.to_string_lossy().to_string());
        info!("Spawning process with args {:?}", exe_args);
        let (process_id, handle) = spawn_process(&exe_args)?;

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

        info!("Suspending process");
        suspend_process(handle)?;

        let result = self.write_to_file(process_id, handle)?;

        info!("Shutting down process");
        resume_process(handle)?;
        terminate_process(handle)?;
        close_handle(handle)?;

        Ok(result)
    }

    pub fn read_block_data(&mut self, block: &SerializedMemoryBlock) -> Result<Vec<u8>> {
        self.file.seek(SeekFrom::Start(block.data_offset))?;
        
        let data_len = self.file.read_u64::<LittleEndian>()? as usize;
        let mut data = vec![0u8; data_len];

        debug!("Reading block at offset {} with size {}", block.data_offset, data_len);

        self.file.read_exact(&mut data)?;
        Ok(data)
    }

    pub fn read_block_as_reader<'a>(&'a mut self, block: &SerializedMemoryBlock) -> Result<impl Read + 'a> {
        self.file.seek(SeekFrom::Start(block.data_offset))?;
        
        let data_len = self.file.read_u64::<LittleEndian>()?;

        debug!("Reading block at offset {} with size {}", block.data_offset, data_len);

        let take_reader = std::io::Read::by_ref(&mut self.file).take(data_len);
        let reader = BufReader::new(take_reader);
        
        Ok(reader)
    }

    fn write_to_file(&mut self, process_id: u32, handle: HANDLE) -> Result<ProcessDump> {

        let mut writer = BufWriter::new(&mut self.file);

        info!("Extracting modules");
        let modules = unsafe { get_modules_map(handle)? };
        
        info!("Extracting active threads");
        let threads = match get_threads(process_id) {
            std::result::Result::Ok(threads) => {
                threads
            },
            Err(err) => {
                error!("{:?}", err);
                vec![]
            },
        };
        
        info!("Extracting module exports");
        let modules_vec = modules.values().cloned().collect::<Vec<_>>();
        let exports = collect_exports(&modules_vec)?;
        let win_version = unsafe { get_windows_version()? };
        let block_iter = MemoryRegionIterator::new(handle);

        let mut dump = ProcessDump::new_with_encode(
            &mut writer,
            win_version,
            threads,
            modules,
            exports,
            block_iter,
            &modules_vec
        )?;

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