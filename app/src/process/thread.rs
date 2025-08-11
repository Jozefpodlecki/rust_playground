use anyhow::*;
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
use windows::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE};
use crate::process::process::close_handle;
use crate::process::thread_context::ThreadContext;

pub fn get_threads(process_id: u32) -> Result<Vec<ThreadContext>> {
    unsafe {
        let mut threads = vec![];
        debug!("Calling CreateToolhelp32Snapshot process id: {}", process_id);
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

        loop {
  
            if thread_entry.th32OwnerProcessID == process_id {
                let thread_id = thread_entry.th32ThreadID;

                let thread_handle = match OpenThread(
                THREAD_GET_CONTEXT | THREAD_SUSPEND_RESUME,
                false, thread_id) {
                    std::result::Result::Ok(handle) => handle,
                    Err(err) => {
                        info!("{:?}", err);
                        if Thread32Next(snapshot, &mut thread_entry).is_err() {
                            break;
                        }

                        continue;
                    },
                };

                SuspendThread(thread_handle);
                let mut context = std::mem::zeroed::<CONTEXT>();
                context.ContextFlags = CONTEXT_FLAGS(CONTEXT_ALL);

                debug!("Getting thread context from thread id: {}", thread_id);
                if GetThreadContext(thread_handle, &mut context).ok().is_some() {
                    let thread = ThreadContext::new(context);
                    threads.push(thread);
                }

                close_handle(thread_handle);
            }

            if Thread32Next(snapshot, &mut thread_entry).is_err() {
                break;
            }
        }
    
        close_handle(snapshot)?;

        Ok(threads)
    }
}