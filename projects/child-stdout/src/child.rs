use core::{mem, ptr::{self, null_mut}, time::Duration};

use ntapi::{ntioapi::IO_STATUS_BLOCK, ntobapi::{NtClose, NtWaitForSingleObject, OBJ_INHERIT}, ntpsapi::{NtAssignProcessToJobObject, NtCreateJobObject, NtCurrentProcess, NtQueryInformationProcess, NtSetInformationJobObject, PROCESS_BASIC_INFORMATION, ProcessBasicInformation}};
use toolkit::{ProcessEnvironmentBlock, Sleeper, U16CStackString, print, println, syscalls::{NtCreateThreadEx, NtReadFile}};
use winapi::{shared::{minwindef::{DWORD, TRUE}, ntdef::{HANDLE, NTSTATUS, OBJ_CASE_INSENSITIVE, OBJ_OPENIF, OBJECT_ATTRIBUTES, UNICODE_STRING}, ntstatus::{STATUS_SUCCESS, STATUS_TIMEOUT}}, um::{errhandlingapi::GetLastError, fileapi::ReadFile, handleapi::{INVALID_HANDLE_VALUE, SetHandleInformation}, jobapi2::{AssignProcessToJobObject, CreateJobObjectW, SetInformationJobObject}, minwinbase::{SECURITY_ATTRIBUTES, STILL_ACTIVE}, namedpipeapi::CreatePipe, processthreadsapi::{CreateProcessW, GetCurrentProcess, PROCESS_INFORMATION, STARTUPINFOW}, winbase::{DETACHED_PROCESS, HANDLE_FLAG_INHERIT, STARTF_USESTDHANDLES}, winnt::{JOB_OBJECT_ALL_ACCESS, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation, LARGE_INTEGER, THREAD_ALL_ACCESS}}};

#[derive(Debug, PartialEq)]
pub enum ClientStatus {
    Alive,
    Dead,
    Unknown,
}

#[derive(Debug)]
pub enum ClientError {
    CreateProcessFailed(u32),
    SpawnLoggerFailed(u32),
    CreateJobObjectFailed,
    SetJobInformationFailed,
    NtAssignProcessToJobObjectFailed,
    WaitTimeout,
    WaitFailed
}

pub struct ClientProcess {
    pub process_handle: *mut winapi::ctypes::c_void,
    pub process_id: u32,
    pub job_handle: *mut winapi::ctypes::c_void,
    pub stdout_read: HANDLE
}

impl ClientProcess {
    
    pub fn create() -> Result<Self, ClientError> {
        unsafe {
            let job_handle = Self::create_and_configure_job()?;

            let peb = ProcessEnvironmentBlock::current_process();
            let executable_path = peb.executable_path().to_owned::<260>().unwrap();
            let mut cmd_line = executable_path;
            cmd_line.push_str(" child");

            let mut stdout_read: HANDLE = INVALID_HANDLE_VALUE;
            let mut stdout_write: HANDLE = INVALID_HANDLE_VALUE;
            let mut sa: SECURITY_ATTRIBUTES = mem::zeroed();
            sa.nLength = mem::size_of_val(&sa) as u32;
            sa.bInheritHandle = TRUE;
            sa.lpSecurityDescriptor = ptr::null_mut();

            let result = CreatePipe(
                &mut stdout_read,
                &mut stdout_write,
                &mut sa,
                0,
            );

            if result == 0 {
                let error = GetLastError();
                return Err(ClientError::CreateProcessFailed(error))
            }

            let result = SetHandleInformation(
                stdout_read,
                HANDLE_FLAG_INHERIT,
                0,
            );

            if result == 0 {
                let error = GetLastError();
                return Err(ClientError::CreateProcessFailed(error))
            }

            let mut startup_info: STARTUPINFOW = mem::zeroed();
            startup_info.dwFlags = STARTF_USESTDHANDLES;
            startup_info.hStdOutput = stdout_write;
            startup_info.hStdError = stdout_write;
            startup_info.cb = mem::size_of::<STARTUPINFOW>() as u32;
            let mut process_info: PROCESS_INFORMATION = mem::zeroed();

            let result = CreateProcessW(
                ptr::null_mut(),
                cmd_line.as_mut_ptr(),
                ptr::null_mut(),
                ptr::null_mut(),
                TRUE,
                DETACHED_PROCESS,
                ptr::null_mut(),
                ptr::null_mut(),
                &mut startup_info,
                &mut process_info,
            );

            if result == 0 {
                let error = GetLastError();
                Err(ClientError::CreateProcessFailed(error))
            } else {
                NtClose(process_info.hThread);
                NtClose(stdout_write);

                Ok(Self {
                    job_handle,
                    process_handle: process_info.hProcess,
                    process_id: process_info.dwProcessId,
                    stdout_read
                })
            }
        }
    }

    fn create_and_configure_job() -> Result<*mut winapi::ctypes::c_void, ClientError> {
        unsafe {
            let mut name = U16CStackString::<30>::from_str("Global\\ChildStdOutJob").unwrap();
            let mut handle: *mut winapi::ctypes::c_void = null_mut();
            let mut unicode_string = UNICODE_STRING {
                Length: name.len() as u16 * 2,
                MaximumLength: name.len() as u16 * 2 + 2,
                Buffer: name.as_mut_ptr(),
            };
           
            let mut object_attributes: OBJECT_ATTRIBUTES = core::mem::zeroed();
            object_attributes.Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as u32;
            object_attributes.RootDirectory = 0x80 as _;
            object_attributes.ObjectName = &mut unicode_string;
            object_attributes.Attributes = OBJ_OPENIF;
            
            // let status = ntapi::ntpsapi::NtCreateJobObject(&mut handle, JOB_OBJECT_ALL_ACCESS, &mut object_attributes);
            // println!("{status:X}");
            let handle = CreateJobObjectW(ptr::null_mut(), name.as_ptr());
            // if status != STATUS_SUCCESS {
            //     return Err(ClientError::CreateJobObjectFailed);
            // }

            let mut job_info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = mem::zeroed();
            job_info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

            let status = NtSetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                &mut job_info as *mut _ as *mut _,
                mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32);

            if status != STATUS_SUCCESS {
                NtClose(handle);
                return Err(ClientError::SetJobInformationFailed);
            }

            let status = NtAssignProcessToJobObject(handle, NtCurrentProcess);
            
            if status != STATUS_SUCCESS {
                return Err(ClientError::NtAssignProcessToJobObjectFailed);
            }

            Ok(handle)
        }
    }
    
    extern "system" fn logger_entry(stdout_read: *mut winapi::ctypes::c_void) {
        unsafe {

            loop {
                let mut buffer = [0; 1024];
                let mut io_status = core::mem::zeroed::<IO_STATUS_BLOCK>();
                let status = NtReadFile(
                    stdout_read,
                    null_mut(),
                    None,
                    null_mut(),
                    &mut io_status,
                    buffer.as_mut_ptr() as _,
                    buffer.len() as _,
                    null_mut(),
                    null_mut(),
                );

                if status >= 0 {
                    let bytes_read = io_status.Information as usize;
                    if let Ok(output) = core::str::from_utf8(&buffer[..bytes_read as usize]) {
                        print!("{}", output);
                    }
                }
            }
        }
    }

    pub fn spawn_logger(&self) -> Result<HANDLE, ClientError> {
        let mut handle: HANDLE = ptr::null_mut();

        let status = unsafe {
            NtCreateThreadEx(
                &mut handle,
                THREAD_ALL_ACCESS,
                ptr::null_mut(),
                NtCurrentProcess,
                Self::logger_entry as *mut _,
                self.stdout_read,
                0,
                0,
                0,
                0,
                ptr::null_mut(),
            )
        };

        if status != STATUS_SUCCESS {
            return Err(ClientError::SpawnLoggerFailed(status as _));
        }

        Ok(handle)
    }

    pub fn check_status(&self) -> ClientStatus {
        unsafe {
            let mut pbi: PROCESS_BASIC_INFORMATION = mem::zeroed();
            let mut return_length = 0;
            // let result = winapi::um::processthreadsapi::GetExitCodeProcess(
            //     self.process_handle,
            //     &mut exit_code,
            // );

            let status = NtQueryInformationProcess(
                self.process_handle,
                ProcessBasicInformation,
                &mut pbi as *mut _ as *mut _,
                mem::size_of::<PROCESS_BASIC_INFORMATION>() as u32,
                &mut return_length,
            );

            if status != STATUS_SUCCESS {
                return ClientStatus::Unknown;
            }

            if pbi.ExitStatus != STILL_ACTIVE as _ {
                return ClientStatus::Dead;
            }

            ClientStatus::Alive
        }
    }

    pub fn is_alive(&self) -> bool {
        self.check_status() == ClientStatus::Alive
    }

    pub fn wait_for_process(&self, timeout: Duration) -> Result<(), ClientError> {
        unsafe {
            let timeout_ms = timeout.as_millis();
            let mut timeout_nt: LARGE_INTEGER = mem::zeroed();
            *timeout_nt.QuadPart_mut() = -(timeout_ms as i64) * 10_000;
            
            let status = NtWaitForSingleObject(
                self.process_handle,
                0,
                &mut timeout_nt,
            );
            
            if status == 0 {
                Ok(())
            } else if status == STATUS_TIMEOUT as i32 {
                Err(ClientError::WaitTimeout)
            } else {
                Err(ClientError::WaitFailed)
            }
        }
    }
}

impl Drop for ClientProcess {
    fn drop(&mut self) {
        unsafe {
            if !self.process_handle.is_null() {
                NtClose(self.process_handle);
            }
        }
    }
}