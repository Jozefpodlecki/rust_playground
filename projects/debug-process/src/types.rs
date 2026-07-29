use core::time::Duration;

use ntapi::{ntapi_base::CLIENT_ID, ntdbg::*, };

use crate::{error::DebuggerError, event::{DebugEvent, DebugEventImpl}};


#[allow(non_snake_case)]
#[repr(C)]
pub struct DBGUI_WAIT_STATE_CHANGE {
    pub NewState: DBG_STATE,
    pub AppClientId: CLIENT_ID,
    pub StateInfo: DBGUI_STATE_INFO,
}

#[allow(non_snake_case)]
#[allow(nonstandard_style)]
pub union DBGUI_STATE_INFO {
    pub Exception: DBGKM_EXCEPTION,
    pub CreateThread: DBGUI_CREATE_THREAD,
    pub CreateProcessInfo: DBGUI_CREATE_PROCESS,
    pub ExitThread: DBGKM_EXIT_THREAD,
    pub ExitProcess: DBGKM_EXIT_PROCESS,
    pub LoadDll: DBGKM_LOAD_DLL,
    pub UnloadDll: DBGKM_UNLOAD_DLL,
}

pub trait Debugger {
    fn attach(&mut self, pid: u32) -> Result<(), DebuggerError>;
    fn wait_for_event(&mut self, option: WaitOptions) -> Result<DebugEvent, DebuggerError>;
    fn continue_event(&self, tid: u32, status: ContinueStatus) -> Result<(), DebuggerError>;
}


#[repr(u32)]
pub enum ContinueStatus {
    ExceptionHandled = 0x00010001,
    Continue = 0x00010002
}

#[derive(Default)]
pub struct WaitOptions {
    pub timeout: Duration,
}

impl WaitOptions {
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            timeout
        }
    }
}
