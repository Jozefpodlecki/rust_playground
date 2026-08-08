use core::{fmt, mem::{self, zeroed}, ptr::{self, null_mut}, time::Duration};

use ntapi::{ntapi_base::CLIENT_ID, ntobapi::{NtClose, NtWaitForSingleObject}, ntpsapi::NtOpenProcess};
use toolkit::{ProcessEnvironmentBlock, ProcessQuerier, Sleeper, U8CStackString, U16CStackString, println, rand::Rng};
use winapi::{shared::{minwindef::{FALSE, TRUE}, ntdef::{HANDLE, NTSTATUS, OBJECT_ATTRIBUTES}}, um::{errhandlingapi::GetLastError, handleapi::{INVALID_HANDLE_VALUE, SetHandleInformation}, jobapi2::{AssignProcessToJobObject, CreateJobObjectW, SetInformationJobObject}, memoryapi::{CreateFileMappingW, FILE_MAP_ALL_ACCESS, MapViewOfFile, UnmapViewOfFile}, minwinbase::SECURITY_ATTRIBUTES, namedpipeapi::CreatePipe, processthreadsapi::{CreateProcessW, GetCurrentProcess, PROCESS_INFORMATION, STARTUPINFOW}, synchapi::{CreateEventW, CreateMutexW, ReleaseMutex}, winbase::{DETACHED_PROCESS, HANDLE_FLAG_INHERIT, STARTF_USESTDHANDLES}, winnt::{JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation, LARGE_INTEGER, PAGE_READWRITE, PROCESS_ALL_ACCESS}}};

use crate::{error::{ServerError, SharedMemoryError}, shared::{SHARED_MEMORY_SIZE, SharedMemory}, client_info::{ClientInfo, ClientStatus}};


pub struct Server {
    shared_memory: SharedMemory,
    _job_handle: *mut winapi::ctypes::c_void,
    client_info: ClientInfo,
    _mapping_handle: *mut winapi::ctypes::c_void,
}

pub struct ServerOptions {
    pub spawn_client: bool
}

impl Server {
    pub fn new(options: ServerOptions) -> Result<Self, ServerError> {
        let mapping_handle = Self::create_file_mapping()?;
        let view_ptr = Self::map_view(mapping_handle)?;
        let mutex_handle = Self::create_mutex()?;
        let event_handle = Self::create_event()?;
        let job_handle = Self::create_and_configure_job()?;

        let mut shared_memory = SharedMemory::new(view_ptr, mutex_handle, event_handle);
        shared_memory.init();


        let client_info = if options.spawn_client {
            Self::create_child_process()?
        } else {
            Self::query_child_process()?
        };

        client_info.spawn_logger().map_err(|err| ServerError::CreateProcessFailed(err as _))?;

        Ok(Self {
            shared_memory,
            _job_handle: job_handle,
            client_info,
            _mapping_handle: mapping_handle,
        })
    }

    fn create_file_mapping() -> Result<*mut winapi::ctypes::c_void, ServerError> {
        unsafe {
            let name = U16CStackString::<30>::from_str("Local\\SharedMemory").unwrap();
            let handle = CreateFileMappingW(
                INVALID_HANDLE_VALUE,
                ptr::null_mut(),
                PAGE_READWRITE,
                0,
                SHARED_MEMORY_SIZE,
                name.as_ptr(),
            );

            if handle.is_null() {
                let error = GetLastError();
                Err(ServerError::CreateFileMappingFailed(error))
            } else {
                Ok(handle)
            }
        }
    }

    fn map_view(handle: *mut winapi::ctypes::c_void) -> Result<*mut winapi::ctypes::c_void, ServerError> {
        unsafe {
            let view = MapViewOfFile(
                handle,
                FILE_MAP_ALL_ACCESS,
                0,
                0,
                SHARED_MEMORY_SIZE as usize,
            );
            if view.is_null() {
                let error = GetLastError();
                NtClose(handle);
                Err(ServerError::MapViewOfFileFailed(error))
            } else {
                Ok(view)
            }
        }
    }

    fn create_mutex() -> Result<*mut winapi::ctypes::c_void, ServerError> {
        unsafe {
            let name = U16CStackString::<30>::from_str("Global\\SharedMemoryMutex").unwrap();
            let handle = CreateMutexW(ptr::null_mut(), FALSE, name.as_ptr());
            if handle.is_null() {
                Err(ServerError::CreateMutexFailed)
            } else {
                Ok(handle)
            }
        }
    }

    fn create_event() -> Result<*mut winapi::ctypes::c_void, ServerError> {
        unsafe {
            let name = U16CStackString::<30>::from_str("Global\\SharedMemoryEvent").unwrap();
            let handle = CreateEventW(ptr::null_mut(), FALSE, FALSE, name.as_ptr());
            if handle.is_null() {
                Err(ServerError::CreateEventFailed)
            } else {
                Ok(handle)
            }
        }
    }

    fn create_and_configure_job() -> Result<*mut winapi::ctypes::c_void, ServerError> {
        unsafe {
            let name = U16CStackString::<30>::from_str("Global\\SharedMemoryJob").unwrap();
            let handle = CreateJobObjectW(ptr::null_mut(), name.as_ptr());
            if handle.is_null() {
                return Err(ServerError::CreateJobObjectFailed);
            }

            let mut job_info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = mem::zeroed();
            job_info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

            let result = SetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                &mut job_info as *mut _ as *mut _,
                mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            );

            if result == 0 {
                NtClose(handle);
                return Err(ServerError::SetJobInformationFailed);
            }

            let result = AssignProcessToJobObject(handle, GetCurrentProcess());

            if result == 0 {
                return Err(ServerError::AssignProcessToJobObjectFailed);
            }

            Ok(handle)
        }
    }
    
    fn query_child_process() -> Result<ClientInfo, ServerError> {
        loop {
            let peb = ProcessEnvironmentBlock::current_process();
            let file_name = peb.executable_path().file_name();
            let name = U8CStackString::<50>::from_utf16_lossy(file_name.as_slice());
            match ProcessQuerier::find_process_by_name(&name) {
                Some(pid) => {
                    println!("Found client, waiting 5s");
                    let handle = Self::open_process_handle(pid).unwrap();
                    return Ok(ClientInfo {
                        process_handle: handle,
                        process_id: pid,
                        stdout_read: null_mut()
                    })
                },
                None => {
                    println!("Could not find client, waiting 5s");
                    Sleeper::sleep(5000);
                },
            }
        }

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
                PROCESS_ALL_ACCESS,
                &mut object_attributes,
                &mut client_id,
            );

            if status < 0 {
                return Err(status);
            }

            Ok(process_handle)
        }
    }

    fn create_child_process() -> Result<ClientInfo, ServerError> {
        unsafe {
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
                return Err(ServerError::CreateProcessFailed(error))
            }
            println!("CreatePipe {:p}", stdout_read);
            let result = SetHandleInformation(
                stdout_read,
                HANDLE_FLAG_INHERIT,
                0,
            );

            if result == 0 {
                let error = GetLastError();
                return Err(ServerError::CreateProcessFailed(error))
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
                Err(ServerError::CreateProcessFailed(error))
            } else {
                NtClose(process_info.hThread);
                NtClose(stdout_write);

                Ok(ClientInfo {
                    process_handle: process_info.hProcess,
                    process_id: process_info.dwProcessId,
                    stdout_read
                })
            }
        }
    }

    pub fn run(&mut self) -> Result<(), ServerError> {
        let mut rng = Rng::new();

        loop {
            match self.shared_memory.lock() {
                Ok(_) => {},
                Err(err) => {
                    println!("{err}");
                    Sleeper::sleep(1000);
                    continue;
                },
            };

            let message = rng.rand_str_alnum::<10>();
            // println!("Server: sending message {}", message);
            self.shared_memory.write_message(message.as_str())?;

            self.shared_memory.unlock()?;
            self.shared_memory.signal()?;
            Sleeper::sleep(1000);
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.shared_memory.close();
    }
}