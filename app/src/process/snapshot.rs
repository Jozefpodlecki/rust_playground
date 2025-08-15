use std::collections::HashMap;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;
use anyhow::*;
use log::*;
use windows::Win32::Foundation::HANDLE;

use crate::process::memory::*;
use crate::process::process::*;
use crate::process::thread::get_threads;
use crate::process::thread_context::ThreadContext;
use crate::process::types::*;
use crate::process::utils::*;
use crate::config::LaunchMethod;

pub struct ProcessSnapshot {
    pub win_version: String,
    pub modules: HashMap<String, ProcessModule>,
    pub exports: HashMap<String, Vec<ProcessModuleExport>>,
    pub blocks: MemoryRegionIterator,
    pub threads: Vec<ThreadContext>
}

impl Drop for ProcessSnapshotter {

    fn drop(&mut self) {
        info!("Shutting down process with handle: {:?}", self.0);
        unsafe {
            if let Err(e) = resume_process(self.0) {
                error!("Failed to resume process: {:?}", e);
            }
            if let Err(e) = terminate_process(self.0) {
                error!("Failed to terminate process: {:?}", e);
            }
            if let Err(e) = close_handle(self.0) {
                error!("Failed to close handle: {:?}", e);
            }
        }
    }
}

pub struct ProcessSnapshotter(HANDLE, u32);

impl ProcessSnapshotter {

    pub fn run(
        exe_path: &Path,
        exe_args: &[String],
        strategy: LaunchMethod) -> Result<Self> {
       
        let (handle, process_id) = unsafe {
            let mut exe_args = exe_args.to_vec();
            exe_args.insert(0, exe_path.to_string_lossy().to_string());
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
            (handle, process_id)
            // get_snapshot(process_id)?
        };

        Ok(Self(handle, process_id))
    }

    pub fn get_snapshot(&self) -> Result<ProcessSnapshot> {

        info!("Extracting modules");
        let modules = unsafe { get_modules_map(self.0)? };
        
        info!("Extracting active threads");
        let threads = match get_threads(self.1) {
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
        let blocks = MemoryRegionIterator::new(self.0);

        let snapshot = ProcessSnapshot {
            win_version,
            blocks,
            exports,
            modules,
            threads
        };

        Ok(snapshot)
    }
}