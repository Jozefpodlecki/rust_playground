#![allow(unsafe_op_in_unsafe_fn)]

use std::ffi::{c_void, OsString};
use std::fmt;
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::os::windows::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;
use anyhow::*;
use log::info;

mod modules;
mod utils;
mod process;
mod dump;
mod memory;
mod types;

pub use dump::ProcessDumpResult;

use crate::process_dumper::memory::*;
use crate::process_dumper::process::*;
use crate::process_dumper::utils::match_module;
use crate::types::RunArgs;

pub struct ProcessDumper;

impl ProcessDumper {
    pub fn new() -> Self {
        Self
    }

    pub fn run_or_get_cached(&self, args: &RunArgs) -> Result<ProcessDumpResult> {
        
        let file_name = Path::new(&args.exe_path).file_stem().unwrap().to_string_lossy();
        let output_bin_path = format!("{file_name}.bin");
        let output_bin_path = args.output_path.join(output_bin_path);

        if output_bin_path.exists() {
            let result = ProcessDumpResult::decode_from_file(&output_bin_path)?;
            return Ok(result);
        };
        
        let result = unsafe { self.run_inner(&args.exe_args, args.addr_offset)? };

        result.encode_to_file(&output_bin_path)?;
        
        Ok(result)
    }

    unsafe fn run_inner(&self, exe_args: &[String], addr_offset: Option<u64>) -> Result<ProcessDumpResult> {

        let handle = spawn_process(exe_args)?;

        sleep(Duration::from_secs(1));

        let main_module = get_main_module(handle)?.unwrap();
        let address = addr_offset.map(|pr| main_module.base + pr);
        let wait_interval = Duration::from_secs(1);

        // if let Some(address) = address {
        //     info!("Monitoring address 0x{address:X} 0x{:X}", addr_offset.unwrap());
        //     monitor_address(handle, address, wait_interval)?;
        // }

        info!("Sleeping for 60s");
        sleep(Duration::from_secs(60));
        info!("Sleeping for 60s");
        sleep(Duration::from_secs(60));

        suspend_process(handle)?;

        let result = ProcessDumpResult::new(handle)?;

        resume_process(handle)?;
        terminate_process(handle)?;
        close_handle(handle)?;

        Ok(result)
    }
}