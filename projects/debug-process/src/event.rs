
use core::{any::{Any, TypeId}, ops::Deref, ptr::null_mut};

use ntapi::{ntdbg::*, ntpsapi::NtCurrentProcess};
use stack_dst::{Stack, Value, buffers::{ArrayBuf, n}};
use toolkit::{ProcessMemoryReader, println};
use winapi::um::minwinbase::*;

use crate::{error::DebuggerError, types::DBGUI_WAIT_STATE_CHANGE};

pub type Ptr32 = ArrayBuf<*const (), n::U32>;
pub type InnerDebugEvent = Value<dyn DebugEventImpl, Ptr32>;

pub struct DebugEvent(InnerDebugEvent);

impl Deref for DebugEvent {
    type Target = dyn DebugEventImpl;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl DebugEvent {
    pub fn from_event<U: DebugEventImpl + 'static>(val: U) -> Result<Self, DebuggerError> {
        InnerDebugEvent::new_stable(val, |v| v as &dyn DebugEventImpl)
            .map_err(|event| DebuggerError::EventTooLarge {
                code: event.event_code(),
                size: size_of::<U>()
            })
            .map(Self)    }

    pub fn as_event<T: 'static>(&self) -> Option<&T> {
        unsafe {
            let t = TypeId::of::<T>();
            let concrete = self.0.deref().type_id();

            if t == concrete {
                let value = &*(self as *const dyn Any as *const T);
                return Some(value)
            }
           
            None
        }
    }
}

pub trait DebugEventImpl: Any {
    fn event_code(&self) -> u32;
    fn process_id(&self) -> u32;
    fn thread_id(&self) -> u32;
    fn name(&self) -> &'static str;
}

macro_rules! define_event {
    (
        $name:ident,
        $info_type:ty,
        $code:expr
    ) => {
        pub struct $name {
            pub info: $info_type,
            pub pid: u32,
            pub tid: u32,
        }

        impl DebugEventImpl for $name {
            fn event_code(&self) -> u32 { $code }
            fn process_id(&self) -> u32 { self.pid }
            fn thread_id(&self) -> u32 { self.tid }
            fn name(&self) -> &'static str { stringify!($name) }
        }
    };

    (
        $name:ident,
        Void
    ) => {
        pub struct $name {
            pub code: u32,
            pub pid: u32,
            pub tid: u32,
        }

        impl DebugEventImpl for $name {
            fn event_code(&self) -> u32 { self.code }
            fn process_id(&self) -> u32 { self.pid }
            fn thread_id(&self) -> u32 { self.tid }
            fn name(&self) -> &'static str { stringify!($name) }
        }
    };
}

macro_rules! event_factory {
    ($event:expr, $struct:ident, $field:ident) => {{
        let info = unsafe { *$event.u.$field() };
        let val = $struct {
            info,
            pid: $event.dwProcessId,
            tid: $event.dwThreadId,
        };
        DebugEvent::from_event(val)
    }};
}


macro_rules! event_factory {
    ($event:expr, $struct:ident, $field:ident) => {{
        let info = unsafe { *$event.u.$field() };
        let val = $struct {
            info,
            pid: $event.dwProcessId,
            tid: $event.dwThreadId,
        };
        DebugEvent::from_event(val)
    }};
}


define_event!(ExceptionEvent, EXCEPTION_DEBUG_INFO, EXCEPTION_DEBUG_EVENT);
define_event!(ThreadEvent, CREATE_THREAD_DEBUG_INFO, CREATE_THREAD_DEBUG_EVENT);
define_event!(CreateProcessEvent, CREATE_PROCESS_DEBUG_INFO, CREATE_PROCESS_DEBUG_EVENT);
define_event!(ExitThreadEvent, EXIT_THREAD_DEBUG_INFO, EXIT_THREAD_DEBUG_EVENT);
define_event!(ExitProcessEvent, EXIT_PROCESS_DEBUG_INFO, EXIT_PROCESS_DEBUG_EVENT);
define_event!(LoadDllEvent, LOAD_DLL_DEBUG_INFO, LOAD_DLL_DEBUG_EVENT);
define_event!(UnloadDllEvent, UNLOAD_DLL_DEBUG_INFO, UNLOAD_DLL_DEBUG_EVENT);
define_event!(DebugStringEvent, OUTPUT_DEBUG_STRING_INFO, OUTPUT_DEBUG_STRING_EVENT);
define_event!(RipEvent, RIP_INFO, RIP_EVENT);
define_event!(BreakpointEvent, EXCEPTION_DEBUG_INFO, DbgBreakpointStateChange);
define_event!(SingleStepEvent, EXCEPTION_DEBUG_INFO, DbgSingleStepStateChange);
define_event!(VoidEvent, Void);

pub struct DebugEventFactory;

impl DebugEventFactory {
    pub fn from(event: DEBUG_EVENT) -> Result<DebugEvent, DebuggerError> {

        match event.dwDebugEventCode {
            EXCEPTION_DEBUG_EVENT => event_factory!(event, ExceptionEvent, Exception),
            CREATE_THREAD_DEBUG_EVENT => event_factory!(event, ThreadEvent, CreateThread),
            CREATE_PROCESS_DEBUG_EVENT => event_factory!(event, CreateProcessEvent, CreateProcessInfo),
            EXIT_THREAD_DEBUG_EVENT => event_factory!(event, ExitThreadEvent, ExitThread),
            EXIT_PROCESS_DEBUG_EVENT => event_factory!(event, ExitProcessEvent, ExitProcess),
            LOAD_DLL_DEBUG_EVENT => event_factory!(event, LoadDllEvent, LoadDll),
            UNLOAD_DLL_DEBUG_EVENT => event_factory!(event, UnloadDllEvent, UnloadDll),
            OUTPUT_DEBUG_STRING_EVENT => event_factory!(event, DebugStringEvent, DebugString),
            RIP_EVENT => event_factory!(event, RipEvent, RipInfo),
            code => {
                let val = VoidEvent {
                    code,
                    pid: event.dwProcessId,
                    tid: event.dwThreadId,
                };
                DebugEvent::from_event(val)
            }
        }
    }

    #[allow(non_upper_case_globals)]
    pub fn from_nt(state: DBGUI_WAIT_STATE_CHANGE) -> Result<DebugEvent, DebuggerError> {
        match state.NewState {
            DbgExceptionStateChange => {
                let nt_info = unsafe { state.StateInfo.Exception };
                let info = EXCEPTION_DEBUG_INFO {
                    ExceptionRecord: nt_info.ExceptionRecord,
                    dwFirstChance: nt_info.FirstChance,
                };
                let val = ExceptionEvent {
                    info,
                    pid: state.AppClientId.UniqueProcess as u32,
                    tid: state.AppClientId.UniqueThread as u32,
                };
                DebugEvent::from_event(val)
            }
            DbgSingleStepStateChange => {
                let nt_info = unsafe { state.StateInfo.Exception };
                let info = EXCEPTION_DEBUG_INFO {
                    ExceptionRecord: nt_info.ExceptionRecord,
                    dwFirstChance: nt_info.FirstChance,
                };
                let val = SingleStepEvent {
                    info,
                    pid: state.AppClientId.UniqueProcess as u32,
                    tid: state.AppClientId.UniqueThread as u32,
                };
                DebugEvent::from_event(val)
            }
            DbgBreakpointStateChange => {
                 let nt_info = unsafe { state.StateInfo.Exception };
                 let info = EXCEPTION_DEBUG_INFO {
                    ExceptionRecord: nt_info.ExceptionRecord,
                    dwFirstChance: nt_info.FirstChance,
                };
                let val = BreakpointEvent {
                    info,
                    pid: state.AppClientId.UniqueProcess as u32,
                    tid: state.AppClientId.UniqueThread as u32,
                };
                DebugEvent::from_event(val)
            }
            DbgCreateThreadStateChange => {
                let nt_info = unsafe { state.StateInfo.CreateThread };
                let info = CREATE_THREAD_DEBUG_INFO {
                    hThread: nt_info.HandleToThread,
                    lpThreadLocalBase: null_mut(),
                    lpStartAddress: Some(unsafe { core::mem::transmute(nt_info.NewThread.StartAddress) }),
                };
                let val = ThreadEvent {
                    info,
                    pid: state.AppClientId.UniqueProcess as u32,
                    tid: state.AppClientId.UniqueThread as u32,
                };
                DebugEvent::from_event(val)
            }
            DbgCreateProcessStateChange => {
                let nt_info = unsafe { state.StateInfo.CreateProcessInfo };
                let info = CREATE_PROCESS_DEBUG_INFO {
                    hFile: nt_info.NewProcess.FileHandle,
                    hProcess: nt_info.HandleToProcess,
                    hThread: nt_info.HandleToThread,
                    lpBaseOfImage: nt_info.NewProcess.BaseOfImage,
                    dwDebugInfoFileOffset: nt_info.NewProcess.DebugInfoFileOffset,
                    nDebugInfoSize: nt_info.NewProcess.DebugInfoSize,
                    lpThreadLocalBase: nt_info.NewProcess.InitialThread.StartAddress,
                    lpStartAddress: unsafe { core::mem::transmute(nt_info.NewProcess.InitialThread.StartAddress) },
                    lpImageName: null_mut(),
                    fUnicode: 0,
                };
                let val = CreateProcessEvent {
                    info,
                    pid: state.AppClientId.UniqueProcess as u32,
                    tid: state.AppClientId.UniqueThread as u32,
                };
                DebugEvent::from_event(val)
            }
            DbgExitThreadStateChange => {
                let nt_info = unsafe { state.StateInfo.ExitThread };
                let info = EXIT_THREAD_DEBUG_INFO {
                    dwExitCode: nt_info.ExitStatus as _,
                };
                let val = ExitThreadEvent {
                    info,
                    pid: state.AppClientId.UniqueProcess as u32,
                    tid: state.AppClientId.UniqueThread as u32,
                };
                DebugEvent::from_event(val)
            }
            DbgExitProcessStateChange => {
                let nt_info = unsafe { state.StateInfo.ExitProcess };
                let info = EXIT_PROCESS_DEBUG_INFO {
                    dwExitCode: nt_info.ExitStatus as _,
                };
                let val = ExitProcessEvent {
                    info,
                    pid: state.AppClientId.UniqueProcess as u32,
                    tid: state.AppClientId.UniqueThread as u32,
                };
                DebugEvent::from_event(val)
            }
            DbgLoadDllStateChange => {
                let nt_info = unsafe { state.StateInfo.LoadDll };
                let info = LOAD_DLL_DEBUG_INFO {
                    hFile: nt_info.FileHandle,
                    lpBaseOfDll: nt_info.BaseOfDll,
                    dwDebugInfoFileOffset: nt_info.DebugInfoFileOffset,
                    nDebugInfoSize: nt_info.DebugInfoSize,
                    lpImageName: null_mut(),
                    fUnicode: 0,
                };
                let val = LoadDllEvent {
                    info,
                    pid: state.AppClientId.UniqueProcess as u32,
                    tid: state.AppClientId.UniqueThread as u32,
                };
                DebugEvent::from_event(val)
            }
            DbgUnloadDllStateChange => {
                let nt_info = unsafe { state.StateInfo.UnloadDll };
                let info = UNLOAD_DLL_DEBUG_INFO {
                    lpBaseOfDll: nt_info.BaseAddress,
                };
                let val = UnloadDllEvent {
                    info,
                    pid: state.AppClientId.UniqueProcess as u32,
                    tid: state.AppClientId.UniqueThread as u32,
                };
                DebugEvent::from_event(val)
            }
            _ => {
                let val = VoidEvent {
                    code: state.NewState as u32,
                    pid: state.AppClientId.UniqueProcess as u32,
                    tid: state.AppClientId.UniqueThread as u32,
                };
                DebugEvent::from_event(val)
            }
        }
    }
}