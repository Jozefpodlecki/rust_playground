use core::{arch::{asm, naked_asm}, mem::{self, zeroed}, ptr::{self, null_mut}, time::Duration};

use ntapi::{ntapi_base::CLIENT_ID, ntdbg::{DEBUG_ALL_ACCESS, DEBUG_KILL_ON_CLOSE, NtCreateDebugObject, NtDebugActiveProcess, NtDebugContinue, NtRemoveProcessDebug, NtWaitForDebugEvent}, ntobapi::NtClose, ntpsapi::{NtCreateThreadEx, NtOpenProcess, NtOpenThread, NtResumeThread, PS_ATTRIBUTE, PS_ATTRIBUTE_LIST}};
use toolkit::{NtDll, ProcessMemoryAlloc, ProcessMemoryReader, ProcessMemoryWriter, ProcessQuerier, println, system_threads};
use winapi::{ctypes::c_void, shared::{ntdef::{HANDLE, OBJECT_ATTRIBUTES}, ntstatus::STATUS_TIMEOUT}, um::winnt::{LARGE_INTEGER, PROCESS_CREATE_THREAD, PROCESS_QUERY_INFORMATION, PROCESS_SUSPEND_RESUME, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE, THREAD_ALL_ACCESS, THREAD_SUSPEND_RESUME}};

use crate::{error::DebuggerError, event::{DebugEvent, DebugEventFactory}, types::{ContinueStatus, DBGUI_WAIT_STATE_CHANGE, Debugger, WaitOptions}, utils::set_remote_process_debugged};

pub const THREAD_CREATE_FLAGS_BYPASS_PROCESS_FREEZE: u32 = 0x40;
pub const THREAD_CREATE_FLAGS_SKIP_LOADER_INIT: u32 = 0x20;
pub const THREAD_CREATE_FLAGS_SKIP_THREAD_ATTACH: u32 = 0x02;

#[derive(Default)]
pub struct NativeWinDebuggerOptions {
    pub store_debug_object_in_teb: bool
}

pub struct NativeWinDebugger {
    breakpoint_thread_handle: *mut winapi::ctypes::c_void,
    teb_debug_object_ptr: *mut HANDLE,
    options: NativeWinDebuggerOptions,
    pid: u32,
    debug_object: HANDLE,
    process_handle: HANDLE,
}

impl Debugger for NativeWinDebugger {
    fn attach(&mut self, pid: u32) -> Result<(), DebuggerError> {
        unsafe {
            let debug_object = self.create_debug_object()?;
            let process_handle = self.open_process_handle(pid)?;
            
            if let Err(err) = self.attach_debugger(process_handle, debug_object) {
                NtClose(process_handle);
                NtClose(debug_object);
                return Err(err);
            }
            
            if let Err(err) = self.inject_breakpoint(process_handle) {
                NtRemoveProcessDebug(process_handle, debug_object);
                NtClose(process_handle);
                NtClose(debug_object);
                return Err(err);
            }
            
            self.pid = pid;
            self.process_handle = process_handle;
            self.debug_object = debug_object;
            
            Ok(())
        }
    }

    fn wait_for_event(&mut self, option: WaitOptions) -> Result<DebugEvent, DebuggerError> {
        unsafe {
            let mut event: DBGUI_WAIT_STATE_CHANGE = zeroed();
            let mut timeout: LARGE_INTEGER = zeroed();

            let timeout_ptr = if option.timeout == Duration::ZERO {
                core::ptr::null_mut()
            } else {
                let nanos = option.timeout.as_nanos();
                *timeout.QuadPart_mut() = -(nanos as i64 / 100);
                &mut timeout
            };

            let status = NtWaitForDebugEvent(
                self.debug_object,
                0,
                timeout_ptr,
                &mut event as *mut _ as *mut _,
            );
            
            if status == STATUS_TIMEOUT {
                return Err(DebuggerError::Timeout);
            }

            if status < 0 {
                return Err(DebuggerError::Wait(status));
            }
            
            let event = DebugEventFactory::from_nt(event)?;

            Ok(event)
        }
    }

    fn continue_event(&self, tid: u32, status: ContinueStatus) -> Result<(), DebuggerError> {
        unsafe { 
            let mut client_id: CLIENT_ID = CLIENT_ID {
                UniqueProcess: self.pid as _,
                UniqueThread: tid as _,
            };

            let status = NtDebugContinue(
                self.debug_object,
                &mut client_id,
                status as _
            );

            if status < 0 {
                return Err(DebuggerError::Continue(status));
            }
        };

        Ok(())
    }
}

impl NativeWinDebugger {
    const DESIRED_ACCESS: u32 = PROCESS_SUSPEND_RESUME | PROCESS_QUERY_INFORMATION | 
        PROCESS_VM_WRITE | PROCESS_VM_READ | PROCESS_VM_OPERATION | 
        PROCESS_CREATE_THREAD;

    pub fn new(options: NativeWinDebuggerOptions) -> Self {
        let teb_ptr: *mut winapi::ctypes::c_void;
        unsafe { core::arch::asm!("mov rax, qword ptr gs:[0x30]", out("rax") teb_ptr) };
        let teb_debug_object_ptr = unsafe { teb_ptr.add(0x16A8) as *mut HANDLE };

        Self {
            breakpoint_thread_handle: null_mut(),
            teb_debug_object_ptr,
            options,
            pid: 0,
            debug_object: null_mut(),
            process_handle: null_mut(),
        }
    }

    pub fn handle(&self) -> HANDLE {
        self.process_handle
    }

    fn create_debug_object(&mut self) -> Result<HANDLE, DebuggerError> {
        unsafe {
            let mut debug_object: HANDLE = core::ptr::null_mut();
            
            let status = NtCreateDebugObject(
                &mut debug_object,
                DEBUG_ALL_ACCESS,
                core::ptr::null_mut(),
                DEBUG_KILL_ON_CLOSE,
            );

            if status < 0 {
                return Err(DebuggerError::CouldNotCreateDebugObject(status));
            }

            if self.options.store_debug_object_in_teb {
                *self.teb_debug_object_ptr = debug_object;
            }

            Ok(debug_object)
        }
    }

    fn open_process_handle(&self, pid: u32) -> Result<HANDLE, DebuggerError> {
        unsafe {
            let mut process_handle: HANDLE = core::ptr::null_mut();
        
            let mut client_id = CLIENT_ID {
                UniqueProcess: pid as _,
                UniqueThread: core::ptr::null_mut(),
            };
            
            let mut object_attributes: OBJECT_ATTRIBUTES = core::mem::zeroed();
            object_attributes.Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as u32;

            let status = NtOpenProcess(
                &mut process_handle,
                Self::DESIRED_ACCESS,
                &mut object_attributes,
                &mut client_id,
            );

            if status < 0 {
                return Err(DebuggerError::CouldNotSpawnProcess(status));
            }

            if process_handle.is_null() {
                return Err(DebuggerError::InvalidHandle);
            }

            Ok(process_handle)
        }
    }

    fn attach_debugger(&self, process_handle: HANDLE, debug_object: HANDLE) -> Result<(), DebuggerError> {

        let status = unsafe { NtDebugActiveProcess(process_handle, debug_object) };
        set_remote_process_debugged(process_handle, 0);
        // let status_1 = unsafe { NtDebugActiveProcess(process_handle, debug_object) };
        // println!("NtDebugActiveProcess 0x{status_1:X}");

        if status < 0 {
            return Err(DebuggerError::CouldNotAttach(status));
        }
        Ok(())
    }

    // DbgUiIssueRemoteBreakin -> DbgUiRemoteBreakin
    #[unsafe(naked)]
    extern "system" fn dbg_ui_remote_break_in() {
        naked_asm!(
            "sub rsp, 0x28",
            "mov rax, qword ptr gs:[0x60]",
            "cmp byte ptr [rax+0x2], 0",
            "jne check_teb",
            "test byte ptr [0x7FFE02D4], 0x2",
            "je terminate",
            
            "check_teb:",
            "mov rax, qword ptr gs:[0x30]",
            "test byte ptr [rax + 0x17EE], 0x20",
            "jne terminate",
            "cmp dword ptr [rcx + 0x1E6100], 0",
            "je breakpoint",
            "mov rax, qword ptr [rcx + 0x1D18A8]",
            "test rax, rax",
            "je breakpoint",
            "call rax",
            
            "breakpoint:",
            "int3",
            "jmp terminate",
            
            "terminate:",
            "mov rcx, 0",
            "xor edx, edx",
            "mov eax, 0x53",
            "syscall",
            "ud2",
            "ret"
        );
    }

     fn prepare_thread_attributes(&self) -> PS_ATTRIBUTE_LIST {
        unsafe {
            let mut client_id: CLIENT_ID = zeroed();
            let mut attr_list: PS_ATTRIBUTE_LIST = zeroed();
            let mut client_id_attr: PS_ATTRIBUTE = zeroed();
            
            client_id_attr.Attribute = 0x00010003;
            client_id_attr.Size = 0x10;
            client_id_attr.u.ValuePtr = &mut client_id as *mut _ as *mut _;
            client_id_attr.ReturnLength = null_mut();
            
            attr_list.TotalLength = 0x28;
            attr_list.Attributes = [client_id_attr];
            
            attr_list
        }
    }

    fn prepare_object_attributes(&self) -> OBJECT_ATTRIBUTES {
        OBJECT_ATTRIBUTES {
            Length: mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
            RootDirectory: ptr::null_mut(),
            ObjectName: ptr::null_mut(),
            Attributes: 0,
            SecurityDescriptor: ptr::null_mut(),
            SecurityQualityOfService: ptr::null_mut(),
        }
    }

    fn inject_shellcode(&self, process_handle: HANDLE) -> Result<*mut c_void, DebuggerError> {
        unsafe {
            let fn_bytes = core::slice::from_raw_parts(
                Self::dbg_ui_remote_break_in as *const u8, 
                90
            );
            let routine_addr = ProcessMemoryAlloc::allocate_and_write_bytes(
                process_handle, 
                &fn_bytes
            )
            .map_err(|_| DebuggerError::MemoryAllocation)?;
            
            Ok(routine_addr)
        }
    }

    fn create_remote_breakpoint_thread(
        &self,
        process_handle: HANDLE,
        routine_addr: *mut c_void,
    ) -> Result<HANDLE, DebuggerError> {
        unsafe {
            const THREAD_CREATE_FLAGS: u32 = THREAD_CREATE_FLAGS_BYPASS_PROCESS_FREEZE
                | THREAD_CREATE_FLAGS_SKIP_LOADER_INIT
                | THREAD_CREATE_FLAGS_SKIP_THREAD_ATTACH;

            let mut obj_attr = self.prepare_object_attributes();
            let mut attr_list = self.prepare_thread_attributes();
            let mut thread_handle: HANDLE = null_mut();
            let ntdll = NtDll::from_remote_process(process_handle);

            let status = NtCreateThreadEx(
                &mut thread_handle,
                THREAD_ALL_ACCESS,
                &mut obj_attr,
                process_handle,
                routine_addr,
                ntdll.base(),
                THREAD_CREATE_FLAGS,
                0,
                0x4000,
                0,
                &mut attr_list,
            );

            if status < 0 {
                return Err(DebuggerError::CouldNotAttach(status));
            }

            Ok(thread_handle)
        }
    }

    fn inject_breakpoint(&mut self, process_handle: HANDLE) -> Result<(), DebuggerError> {
        let routine_addr = self.inject_shellcode(process_handle)?;
        let breakpoint_thread_handle = self.create_remote_breakpoint_thread(process_handle, routine_addr)?;

        self.breakpoint_thread_handle = breakpoint_thread_handle;
        
        Ok(())
    }

    pub fn resume_main_thread(&self) -> Result<(), DebuggerError> {

        unsafe {
            let tid = system_threads(self.pid)
                .map_err(|_| DebuggerError::CouldNotResumeThread(-1))?
                .next()
                .ok_or_else(|| DebuggerError::CouldNotResumeThread(-1))?;

            let mut thread_handle: HANDLE = core::ptr::null_mut();
            let mut client_id = CLIENT_ID {
                UniqueProcess: self.pid as _,
                UniqueThread: tid as _,
            };
            
            let mut object_attributes: OBJECT_ATTRIBUTES = core::mem::zeroed();
            object_attributes.Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as u32;

            let status = NtOpenThread(
                 &mut thread_handle,
                THREAD_SUSPEND_RESUME,
                &mut object_attributes,
                &mut client_id
            );

            if status < 0 {
                return Err(DebuggerError::CouldNotResumeThread(status));
            }

            let status = NtResumeThread(thread_handle, null_mut());

            if status < 0 {
                return Err(DebuggerError::CouldNotResumeThread(status));
            }
        }

        Ok(())
    }
}

impl Drop for NativeWinDebugger {
    fn drop(&mut self) {
        unsafe {
            NtRemoveProcessDebug(self.process_handle, self.debug_object);
            NtClose(self.process_handle);
        }
    }
}