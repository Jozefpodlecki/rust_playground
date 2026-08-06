use core::mem::zeroed;
use core::ptr::null_mut;
use core::{mem, panic::PanicInfo, ptr};
use core::fmt::Write;

use ntapi::ntdbg::DbgUiIssueRemoteBreakin;
use ntapi::ntobapi::{NtClose, NtWaitForSingleObject};
use ntapi::ntpsapi::{NtCreateUserProcess, NtCurrentProcess, NtCurrentProcessId, NtQueryInformationProcess, NtTerminateProcess, PROCESS_BASIC_INFORMATION, ProcessBasicInformation};
use ntapi::ntseapi::{NtAdjustPrivilegesToken, NtOpenProcessToken};
use toolkit::{ProcessEnvironmentBlock, ProcessSpawner, Sleeper, U16CStackString};
use winapi::shared::minwindef::BOOL;
use winapi::shared::ntdef::{LPCWSTR, NTSTATUS, PLUID};
use winapi::shared::ntstatus::STATUS_SUCCESS;
use winapi::um::debugapi::DebugActiveProcess;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::lsalookup::{LSA_OBJECT_ATTRIBUTES, LSA_UNICODE_STRING, PLSA_OBJECT_ATTRIBUTES, PLSA_UNICODE_STRING};
use winapi::um::minwinbase::STILL_ACTIVE;
use winapi::um::ntsecapi::{LSA_HANDLE, PLSA_HANDLE, POLICY_LOOKUP_NAMES};
use winapi::um::securitybaseapi::AdjustTokenPrivileges;
use winapi::um::winbase::{CREATE_NEW_CONSOLE, LookupPrivilegeValueW};
use winapi::um::winnt::{ACCESS_MASK, LARGE_INTEGER, LUID, SE_DEBUG_NAME, SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES};

use ntapi::{ntapi_base::CLIENT_ID, ntdbg::{DEBUG_ALL_ACCESS, DEBUG_KILL_ON_CLOSE, NtCreateDebugObject, NtDebugActiveProcess, NtDebugContinue, NtRemoveProcessDebug, NtWaitForDebugEvent}, ntpsapi::{NtCreateThreadEx, NtOpenProcess, NtOpenThread, NtResumeThread, PS_ATTRIBUTE, PS_ATTRIBUTE_LIST}};
use toolkit::{NtDll, ProcessMemoryAlloc, ProcessMemoryReader, ProcessMemoryWriter, ProcessQuerier, println, system_threads};
use winapi::{ctypes::c_void, shared::{ntdef::{HANDLE, OBJECT_ATTRIBUTES}, ntstatus::STATUS_TIMEOUT}, um::winnt::{PROCESS_CREATE_THREAD, PROCESS_QUERY_INFORMATION, PROCESS_SUSPEND_RESUME, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE, THREAD_ALL_ACCESS, THREAD_SUSPEND_RESUME}};

use winapi::{shared::minwindef::FALSE, um::{processthreadsapi::{CreateProcessW, PROCESS_INFORMATION, STARTUPINFOW}, winbase::DETACHED_PROCESS}};


const DESIRED_ACCESS: u32 = PROCESS_SUSPEND_RESUME | PROCESS_QUERY_INFORMATION | 
    PROCESS_VM_WRITE | PROCESS_VM_READ | PROCESS_VM_OPERATION | 
    PROCESS_CREATE_THREAD;

unsafe extern "system" {
    pub fn LsaLookupPrivilegeValue(
        PolicyHandle: LSA_HANDLE,
        Name: PLSA_UNICODE_STRING,
        Value: PLUID
    ) -> NTSTATUS;
    pub fn  LsaOpenPolicy(
        SystemName: PLSA_UNICODE_STRING,
        ObjectAttributes: PLSA_OBJECT_ATTRIBUTES,
        DesiredAccess: ACCESS_MASK,
        PolicyHandle: PLSA_HANDLE
    ) -> NTSTATUS;
}

pub fn enable_debug_privilege() -> Result<(), NTSTATUS> {
    unsafe {
        let mut token: HANDLE = null_mut();
        let status = NtOpenProcessToken(
            NtCurrentProcess,
            TOKEN_ADJUST_PRIVILEGES,
            &mut token);

        if status < 0 {
            return Err(status);
        }

        let mut luid: LUID = core::mem::zeroed();
        let mut privilege_value = U16CStackString::<30>::from_str(SE_DEBUG_NAME).unwrap();

        let mut uc_str: LSA_UNICODE_STRING = zeroed();
        uc_str.Buffer = privilege_value.as_mut_ptr();
        uc_str.Length = privilege_value.len() as u16 * 2;
        uc_str.MaximumLength = privilege_value.len() as u16 * 2 + 2;

        let mut lsa_handle: LSA_HANDLE = null_mut();
        let mut object_attributes: LSA_OBJECT_ATTRIBUTES = zeroed();
        object_attributes.Length = mem::size_of::<LSA_OBJECT_ATTRIBUTES>() as u32;

        let status = LsaOpenPolicy(
            null_mut(),
            &mut object_attributes,
            POLICY_LOOKUP_NAMES,
            &mut lsa_handle
        );

        if status < 0 {
            return Err(status);
        }

        let status = LookupPrivilegeValueW(
            null_mut(),
            privilege_value.as_mut_ptr(),
            &mut luid);

        let status = LsaLookupPrivilegeValue(
            lsa_handle,
            &mut uc_str,
            &mut luid);

        if status < 0 {
            return Err(status);
        }

        let mut privileges: TOKEN_PRIVILEGES = zeroed();
        privileges.PrivilegeCount = 1;
        privileges.Privileges[0].Luid = luid;
        privileges.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;

        let status = NtAdjustPrivilegesToken(
            token,
            0,
            &mut privileges,
            mem::size_of::<TOKEN_PRIVILEGES>() as u32,
            null_mut(),
            null_mut()
        );

        if status < 0 {
            return Err(status);
        }
    }

    Ok(())
}

pub fn debug_parent(parent_pid: u32) -> Result<*mut c_void, NTSTATUS> {
    unsafe {
        let process_handle: *mut c_void = open_process_handle(parent_pid)?;
        let mut debug_object: HANDLE = core::ptr::null_mut();
        
        let status = NtCreateDebugObject(
            &mut debug_object,
            DEBUG_ALL_ACCESS,
            core::ptr::null_mut(),
            // DEBUG_KILL_ON_CLOSE,
            0
        );

        if status < 0 {
            return Err(status);
        }

        let status = NtDebugActiveProcess(process_handle, debug_object);

        if status < 0 {
            return Err(status);
        }
        
        let status = DbgUiIssueRemoteBreakin(process_handle);

        if status < 0 {
            return Err(status);
        }

        Ok(process_handle)
    }
}

#[derive(Debug)]
pub enum ProcessStatus {
    Alive,
    Unknown,
    Dead(i32)
}

pub fn check_status(process_handle: *mut c_void) -> ProcessStatus {
    unsafe {
        let mut pbi: PROCESS_BASIC_INFORMATION = mem::zeroed();
        let mut return_length = 0;
        
        let status = NtQueryInformationProcess(
            process_handle,
            ProcessBasicInformation,
            &mut pbi as *mut _ as *mut _,
            mem::size_of::<PROCESS_BASIC_INFORMATION>() as u32,
            &mut return_length,
        );

        if status != STATUS_SUCCESS {
            return ProcessStatus::Unknown;
        }

        if pbi.ExitStatus != STILL_ACTIVE as _ {
            return ProcessStatus::Dead(pbi.ExitStatus);
        }

        ProcessStatus::Alive
    }
}

pub fn spawn_child(peb: ProcessEnvironmentBlock, pid: u32) -> Result<*mut winapi::ctypes::c_void, NTSTATUS> {
    unsafe {
        let mut startup_info: STARTUPINFOW = mem::zeroed();
        startup_info.cb = mem::size_of::<STARTUPINFOW>() as u32;
        let mut process_info: PROCESS_INFORMATION = mem::zeroed();
        
        let executable_path = peb.executable_path();  
        let mut command_line = U16CStackString::<200>::from_u16_slice(executable_path.as_slice()).unwrap();
        write!(&mut command_line, " {}", pid).unwrap();

        let result = 
            CreateProcessW(
                ptr::null_mut(),
                command_line.as_mut_ptr(),
                ptr::null_mut(),
                ptr::null_mut(),
                FALSE,
                // DETACHED_PROCESS,
                CREATE_NEW_CONSOLE,
                ptr::null_mut(),
                ptr::null_mut(),
                &mut startup_info,
                &mut process_info
            );

        if result == 0 {
            let error = GetLastError();
            return Err(error as _);
        }

        NtClose(process_info.hThread);

        Ok(process_info.hProcess)
    }
}

struct ThreadArgs {
    pub process_handle: *mut winapi::ctypes::c_void
}

impl ThreadArgs {

}

static mut THREAD_ARGS: ThreadArgs = ThreadArgs {
    process_handle: null_mut()
};

unsafe extern "system" fn wait_for_child_to_finish(args_ptr: *const ThreadArgs) {
    unsafe {
        let args = &*args_ptr;

        let mut delay: LARGE_INTEGER = zeroed();
        *delay.QuadPart_mut() = i64::MIN; 

        let status = NtWaitForSingleObject(args.process_handle, 1, &mut delay);
        println!(" NtWaitForSingleObject {status}");
        // if status < 0 {
        //     return Err(status);
        // }

        NtClose(args.process_handle);
    }
}

pub fn spawn_wait_thread(process_handle: *mut winapi::ctypes::c_void, child_process_handle: *mut winapi::ctypes::c_void) -> Result<(), NTSTATUS> {
    let mut handle: HANDLE = ptr::null_mut();
    unsafe { THREAD_ARGS.process_handle = child_process_handle };

    let status = unsafe {
        NtCreateThreadEx(
            &mut handle,
            THREAD_ALL_ACCESS,
            ptr::null_mut(),
            process_handle,
            wait_for_child_to_finish as _,
            &mut THREAD_ARGS as *mut _ as  _,
            0,
            0,
            0,
            0,
            ptr::null_mut(),
        )
    };

    println!("NtCreateThreadEx {status:X}");
    Ok(())
}

fn open_process_handle(pid: u32) -> Result<HANDLE, NTSTATUS> {
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
            DESIRED_ACCESS,
            &mut object_attributes,
            &mut client_id,
        );

        if status < 0 {
            return Err(status);
        }

        Ok(process_handle)
    }
}