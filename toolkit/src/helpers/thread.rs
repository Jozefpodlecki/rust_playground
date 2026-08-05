use core::iter::Iterator;
use ntapi::ntexapi::{NtQuerySystemInformation, SYSTEM_INFORMATION_CLASS, SYSTEM_PROCESS_INFORMATION, SystemProcessInformation};
use winapi::shared::ntdef::NTSTATUS;
use winapi::shared::ntstatus::{STATUS_INFO_LENGTH_MISMATCH, STATUS_SUCCESS};
use winapi::um::handleapi::CloseHandle;

use crate::println;

const BUFFER_SIZE: usize = 2_000_000;
static mut BUFFER: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

pub fn system_threads(pid: u32) -> Result<impl Iterator<Item = u32>, NTSTATUS> {
    struct SysThreadIter {
        proc_info: *const SYSTEM_PROCESS_INFORMATION,
        pid: u32,
        thread_idx: u32,
        thread_count: u32,
        done: bool,
    }
    
    impl Iterator for SysThreadIter {
        type Item = u32;
        
        fn next(&mut self) -> Option<Self::Item> {
            if self.done {
                return None;
            }
            
            unsafe {
                while !self.proc_info.is_null() && self.thread_idx == 0 {
                    let proc = &*self.proc_info;
                    if proc.UniqueProcessId as u32 == self.pid {
                        self.thread_count = proc.NumberOfThreads;
                        break;
                    }
                    
                    if proc.NextEntryOffset == 0 {
                        self.done = true;
                        return None;
                    }
                    self.proc_info = (self.proc_info as *const u8).add(proc.NextEntryOffset as usize) 
                        as *const SYSTEM_PROCESS_INFORMATION;
                }
                
                if self.proc_info.is_null() {
                    self.done = true;
                    return None;
                }
                
                if self.thread_idx >= self.thread_count {
                    self.done = true;
                    return None;
                }
                
                // Get thread ID
                let proc = &*self.proc_info;
                let thread_ptr = proc.Threads.as_ptr();
                let thread = &*thread_ptr.add(self.thread_idx as usize);
                let tid = thread.ClientId.UniqueThread as u32;

                self.thread_idx += 1;
                
                Some(tid)
            }
        }
    }
    
    unsafe {
        let mut size = 0;
        let status = NtQuerySystemInformation(
            SystemProcessInformation,
            core::ptr::null_mut(),
            0,
            &mut size,
        );
        
        if status != STATUS_SUCCESS && status != STATUS_INFO_LENGTH_MISMATCH {
            return Err(status);
        }
        
        let status = NtQuerySystemInformation(
            SystemProcessInformation,
            BUFFER.as_mut_ptr() as *mut _,
            size,
            &mut size,
        );
        
        if status != STATUS_SUCCESS {
            return Err(status);
        }
        
        Ok(SysThreadIter {
            proc_info: BUFFER.as_ptr() as *const SYSTEM_PROCESS_INFORMATION,
            pid,
            thread_idx: 0,
            thread_count: 0,
            done: false,
        })
    }
}