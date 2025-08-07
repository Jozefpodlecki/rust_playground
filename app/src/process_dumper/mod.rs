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
use log::*;
use winapi::um::winnt::CONTEXT_ALL;
use windows::Win32::System::Diagnostics::Debug::{GetThreadContext, CONTEXT, CONTEXT_FLAGS};
use windows::Win32::System::Threading::{OpenThread, SuspendThread, THREAD_GET_CONTEXT, THREAD_SUSPEND_RESUME};
use windows::{
    Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Thread32First, Thread32Next, THREADENTRY32,
        TH32CS_SNAPTHREAD,
    }
};


mod utils;
mod process;
mod memory;
mod types;

use object::Object;
use windows::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE};

use crate::process_dumper::memory::*;
use crate::process_dumper::process::*;
use crate::process_dumper::types::*;
use crate::process_dumper::utils::*;
use crate::types::{AppConfig, LaunchMethod};

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
        debug!("Spawning process with args {:?}", exe_args);
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

        debug!("Suspending process");
        suspend_process(handle)?;

        let result = self.write_to_file(process_id, handle)?;

        debug!("Cleanup");
        resume_process(handle)?;
        terminate_process(handle)?;
        close_handle(handle)?;

        Ok(result)
    }

    fn get_threads(process_id: u32) -> Result<Vec<ThreadContext>> {
        unsafe {
            let mut threads = vec![];
            let snapshot: HANDLE = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, process_id)?;

            if snapshot == INVALID_HANDLE_VALUE {
                error!("Failed to create snapshot.");
                return Ok(threads);
            }

            let mut thread_entry = THREADENTRY32 {
                dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
                ..Default::default()
            };

            Thread32First(snapshot, &mut thread_entry)?;

            let mut thread_id = thread_entry.th32ThreadID;
            
            loop {
                let h_thread = OpenThread(
                    THREAD_GET_CONTEXT | THREAD_SUSPEND_RESUME,
                    false, thread_id)?;

                SuspendThread(h_thread);
                let mut context = std::mem::zeroed::<CONTEXT>();
                context.ContextFlags = CONTEXT_FLAGS(CONTEXT_ALL);

                if GetThreadContext(h_thread, &mut context).ok().is_some() {
                    let thread = ThreadContext::new(context);
                    threads.push(thread);
                } else {
                }

                close_handle(h_thread);

                if Thread32Next(snapshot, &mut thread_entry).is_err() {
                    break;
                }
            }
        
            close_handle(snapshot)?;

            Ok(threads)
        }
    }

    pub fn read_block_data(&mut self, block: &SerializedMemoryBlock) -> Result<Vec<u8>> {
        self.file.seek(SeekFrom::Start(block.data_offset))?;
        
        let data_len = self.file.read_u32::<LittleEndian>()? as usize;
        let mut data = vec![0u8; data_len];

        debug!("Reading block at offset {} with size {}", block.data_offset, data_len);

        self.file.read_exact(&mut data)?;
        Ok(data)
    }

    fn write_to_file(&mut self, process_id: u32, handle: HANDLE) -> Result<ProcessDump> {

        let mut writer = BufWriter::new(&mut self.file);

        info!("Extracting modules");
        let modules = unsafe { enumerate_modules(handle)? };
        
        info!("Extracting active threads");
        let threads = match Self::get_threads(process_id) {
            std::result::Result::Ok(threads) => {
                threads
            },
            Err(err) => {
                error!("{:?}", err);
                vec![]
            },
        };
        
        info!("Extracting module exports");
        let mut exports = collect_exports(&modules)?;
        let module_map: HashMap<String, ProcessModule> = modules
            .clone()
            .into_iter()
            .map(|m| (m.file_name.clone(), m))
            .collect();

        let win_version = unsafe { get_windows_version()? };
        write_string(&mut writer, &win_version)?;
        Self::write_modules(&mut writer, &module_map)?;
        Self::write_module_exports(&mut writer, &exports)?;
       
        info!("Extracting memory regions");
        let block_iter = MemoryRegionIterator::new(handle);
        let blocks = Self::write_memory_blocks(&mut writer, block_iter, &modules)?;

        writer.write_all(&threads.len().to_le_bytes())?;
        for thread in threads.iter() {
            thread.encode(&mut writer)?;
        }

        let dump = ProcessDump {
            win_version,
            exports,
            modules: module_map,
            blocks,
            threads
        };

        Ok(dump)
    }

    fn write_modules<W: Write>(writer: &mut W, modules: &HashMap<String, ProcessModule>) -> Result<()> {
        writer.write_all(&(modules.len() as u32).to_le_bytes())?;
        for (name, module) in modules {
            write_string(writer, name)?;
            writer.write_all(&module.order.to_le_bytes())?;
            writer.write_all(&[module.is_dll as u8])?;
            write_string(writer, &module.file_path.to_string_lossy())?;
            write_string(writer, &module.file_name)?;
            writer.write_all(&module.entry_point.to_le_bytes())?;
            writer.write_all(&module.size.to_le_bytes())?;
            writer.write_all(&module.base.to_le_bytes())?;
        }
        Ok(())
    }

    fn write_module_exports<W: Write>(writer: &mut W, exports: &HashMap<String, Vec<ProcessModuleExport>>) -> Result<()> {
        writer.write_all(&(exports.len() as u32).to_le_bytes())?;
        for (module_name, export_list) in exports {
            write_string(writer, module_name)?;
            writer.write_all(&(export_list.len() as u32).to_le_bytes())?;

            for export in export_list {
                write_string(writer, &export.name)?;
                writer.write_all(&export.address.to_le_bytes())?;
            }
        }
        Ok(())
    }

    fn write_memory_blocks<W: Write + Seek>(
        writer: &mut W,
        block_iter: MemoryRegionIterator,
        modules: &[ProcessModule],
    ) -> Result<Vec<SerializedMemoryBlock>> {
        let mut blocks = Vec::new();
        let mut count = 0u32;

        let count_pos = writer.stream_position()?;
        writer.write_all(&count.to_le_bytes())?;

        for block in block_iter {
            let (mut block, data) = block?;
            count += 1;

            block.module_name = match_module(block.base, modules).map(|m| m.file_name.clone());

            let mut serialized = SerializedMemoryBlock {
                data_offset: 0,
                block,
            };

            serialized.encode(writer)?;
            serialized.data_offset = writer.stream_position()?;
            debug!("Writing block at offset {} with size {}", serialized.data_offset, data.len());
            writer.write_all(&(data.len() as u32).to_le_bytes())?;
            writer.write_all(&data)?;

            blocks.push(serialized);
        }

        writer.seek(SeekFrom::Start(count_pos))?;
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