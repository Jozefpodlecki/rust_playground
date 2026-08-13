use core::{mem::zeroed, ptr::null_mut};

use crate::{builder::{AttributesBuilder, environment::EnvironmentBuilder, error::ProcessBuilderError, process_param::ProcessParamsBuilder}, types::ProcessInfo};
use ntapi::ntpsapi::{NtCreateUserProcess, PROCESS_CREATE_FLAGS_SUSPENDED, PS_CREATE_INFO, THREAD_CREATE_FLAGS_CREATE_SUSPENDED};
use toolkit::ProcessEnvironmentBlock;
use winapi::um::handleapi::INVALID_HANDLE_VALUE;


pub struct ProcessBuilder<'a, const ENV_SIZE: usize, const PROCESS_PARAM_SIZE: usize, const ATTR_SIZE: usize> {
    process_desired_access: u32,
    thread_desired_access: u32,
    process_flags: u32,
    thread_flags: u32,
    environment: EnvironmentBuilder<ENV_SIZE>,
    process_params: ProcessParamsBuilder<'a, PROCESS_PARAM_SIZE>,
    attributes: AttributesBuilder<'a, ATTR_SIZE>,
    create_info: PS_CREATE_INFO
}

impl<'a, const ENV_SIZE: usize, const PROCESS_PARAM_SIZE: usize, const ATTR_SIZE: usize> 
    ProcessBuilder<'a, ENV_SIZE, PROCESS_PARAM_SIZE, ATTR_SIZE> 
{
    pub fn new() -> Self {
        Self {
            process_desired_access: 0,
            thread_desired_access: 0,
            process_flags: 0,
            thread_flags: 0,
            environment: EnvironmentBuilder::new(),
            attributes: AttributesBuilder::new(),
            process_params: ProcessParamsBuilder::new(),
            create_info: unsafe { zeroed() }
        }
    }

    pub fn detached(mut self) -> Self {
        self.process_params.set_console_handle(INVALID_HANDLE_VALUE);
        
        self
    }

    pub fn debug(mut self) -> Self {
        self.create_info.u.InitState.InitFlags = 0x2000000F;
        self
    }

    pub fn suspended(mut self) -> Self {
        self.process_desired_access |= PROCESS_CREATE_FLAGS_SUSPENDED;
        self.thread_desired_access |= PROCESS_CREATE_FLAGS_SUSPENDED;
        self.thread_flags |= THREAD_CREATE_FLAGS_CREATE_SUSPENDED;
        self.create_info.u.InitState.InitFlags = 0x20000003;
        self.create_info.u.InitState.AdditionalFileAccess = 0x81;

        self
    }

    pub fn with_parent_pid(mut self, pid: u32) -> Self {
        self.attributes.set_parent_pid(pid);

        self
    }

    pub fn with_absolute_image_path(mut self, path: &'a str) -> Self {
        self.process_params.set_image_path(path);
        self.attributes.set_image_path(path);

        self
    }

    pub fn with_current_directory(mut self, current_dir: &'a str) -> Self {
        self.process_params.set_current_directory(current_dir);

        self
    }

    pub fn with_process_group_from_peb(mut self) -> Self {
        let peb = ProcessEnvironmentBlock::current_process();
        let params = peb.process_params();

        self.process_params.set_process_group(params.ProcessGroupId);

        self
    }

    pub fn with_environment_from_peb(mut self) -> Self {

        let peb = ProcessEnvironmentBlock::current_process();
        let params = peb.process_params();
        self.environment = EnvironmentBuilder::from_raw_parts(params.Environment, params.EnvironmentSize).unwrap();

        self
    }

    pub fn with_command_args(mut self, args: &'a str) -> Self {
        self.process_params.set_command_args(args);

        self
    }

    pub fn spawn(mut self) -> Result<ProcessInfo, ProcessBuilderError> {
        unsafe {
            let mut process_handle = null_mut();
            let mut thread_handle = null_mut();

            self.create_info.Size = size_of::<PS_CREATE_INFO>();
            self.process_params.set_environment(self.environment.as_mut_ptr(), self.environment.len());
            self.process_params.build()?;
            self.attributes.build()?;

            let status = NtCreateUserProcess(
                &mut process_handle,
                &mut thread_handle,
                self.process_desired_access,
                self.thread_desired_access,
                null_mut(),
                null_mut(),
                self.process_flags,
                self.thread_flags,
                self.process_params.as_mut_ptr() as _,
                &mut self.create_info,
                self.attributes.as_mut_ptr()
            );

            if status < 0 {
                let error = match self.create_info.State {
                    1 => ProcessBuilderError::CouldNotOpenFile,
                    2 => ProcessBuilderError::CouldNotCreateSection,
                    3 => ProcessBuilderError::InvalidImageFormat,
                    4 => ProcessBuilderError::MachineMismatch,
                    5 => ProcessBuilderError::ImageFileExecutionOptions,
                    _ => ProcessBuilderError::from(status),
                };
                return Err(error);
            }

            let client_id = self.attributes.client_id();
            let image_info = self.attributes.image_info();
            
            Ok(ProcessInfo {
                pid: client_id.UniqueProcess as u32,
                tid: client_id.UniqueThread as u32,
                entry_point: image_info.TransferAddress,
                peb_addr: self.create_info.u.SuccessState.PebAddressNative,
                section_handle: self.create_info.u.SuccessState.SectionHandle,
                file_handle: self.create_info.u.SuccessState.FileHandle,
                params_addr: self.create_info.u.SuccessState.UserProcessParametersNative,
                output_flags: self.create_info.u.SuccessState.OutputFlags,
                manifest_addr: self.create_info.u.SuccessState.ManifestAddress,
                manifest_size: self.create_info.u.SuccessState.ManifestSize,
            })
        }
    }
}