use core::{mem::zeroed, ptr::null_mut, time::Duration};

use toolkit::system_threads;
use winapi::{shared::{minwindef::BOOL, ntdef::HANDLE }, um::{debugapi::{ContinueDebugEvent, DebugActiveProcess, DebugActiveProcessStop, WaitForDebugEvent}, errhandlingapi::GetLastError, handleapi::CloseHandle, minwinbase::DEBUG_EVENT , processthreadsapi::{OpenThread, ResumeThread}, winbase::INFINITE, winnt::THREAD_SUSPEND_RESUME}};

use crate::{error::DebuggerError, event::{DebugEvent, DebugEventFactory}, types::{ContinueStatus, Debugger, WaitOptions}};

#[derive(Default)]
pub struct WinDebuggerOptions {}

pub struct WinDebugger {
    options: WinDebuggerOptions,
    pid: u32,
    process_handle: HANDLE,
}

impl Debugger for WinDebugger {
    fn attach(&mut self, pid: u32) -> Result<(), DebuggerError> {
        unsafe {
            let ok: BOOL = DebugActiveProcess(pid);

            if ok == 0 {
                let error = GetLastError();
                return Err(DebuggerError::CouldNotAttach(error as i32));
            }

            self.pid = pid;
                
            Ok(())
        }
    }

    fn wait_for_event(&mut self, option: WaitOptions) -> Result<DebugEvent, DebuggerError> {
        unsafe {
            let mut event: DEBUG_EVENT = zeroed();

            let timeout_ms = if option.timeout == Duration::ZERO {
                    INFINITE
                } else {
                    option.timeout.as_millis() as u32
                };
            let ok = WaitForDebugEvent(&mut event, timeout_ms);

            if ok == 0 {
                let error = GetLastError;
                return Err(DebuggerError::Wait(error as i32));
            }

            let event = DebugEventFactory::from(event)?;

            Ok(event)
        }
    }

    fn continue_event(&self, tid: u32, status: ContinueStatus) -> Result<(), DebuggerError> {
        unsafe {
            let result = ContinueDebugEvent(self.pid, tid, status as _);

            if result == 0 {
                let error = GetLastError();
                return Err(DebuggerError::Continue(error as i32));
            }
        }

        Ok(())
    }
}

impl WinDebugger {

    pub fn new(options: WinDebuggerOptions) -> Self {

        Self {
            options,
            pid: 0,
            process_handle: null_mut(),
        }
    }

    pub fn resume_main_thread(&self) -> Result<(), DebuggerError> {
        
        unsafe {
            let tid = system_threads(self.pid).unwrap().next().unwrap();
            let thread_handle = OpenThread(
                THREAD_SUSPEND_RESUME,
                0,
                tid
            );

             if thread_handle.is_null() {
                let error = GetLastError();
                return Err(DebuggerError::CouldNotResumeThread(error as i32));
            }

            let suspend_count = ResumeThread(thread_handle);

            if suspend_count == 0xFFFFFFFF {
                let error = GetLastError();
                return Err(DebuggerError::CouldNotResumeThread(error as i32));
            }
        }

        Ok(())
    }

    pub fn get_tids(&self) -> impl Iterator<Item = u32> {
        system_threads(self.pid).unwrap()
    }

}

impl Drop for WinDebugger {
    fn drop(&mut self) {
        unsafe {
            DebugActiveProcessStop(self.pid);
            CloseHandle(self.process_handle);
        }
    }
}
