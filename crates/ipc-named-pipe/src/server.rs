use core::{mem, ptr::null_mut};

use log::info;
use ntapi::ntioapi::*;
use win_platform::{NtError, syscalls::{NtClose, NtCreateEvent, NtCreateNamedPipeFile, NtFsControlFile, NtReadFile, NtWriteFile}, types::UnicodeString};
use winapi::{shared::{ntdef::{NotificationEvent, OBJ_CASE_INSENSITIVE, OBJECT_ATTRIBUTES}, ntstatus::{STATUS_PIPE_CONNECTED, STATUS_PIPE_LISTENING}}, um::winnt::*};
use win_platform::types::HANDLE;

use crate::{constants::FSCTL_PIPE_LISTEN, error::IpcError, event::Event, types::*, utils::default_pipe_name};

const BUFFER_SIZE: u32 = 4096;

pub struct IpcServer(HANDLE);

// pub struct ObjectAttributes

impl IpcServer {
    pub fn create() -> Result<Self, IpcError> {
        unsafe {
            let mut handle = null_mut();
            let mut io_status_block = core::mem::zeroed::<IO_STATUS_BLOCK>();
            let mut timeout: LARGE_INTEGER = core::mem::zeroed();
            *timeout.QuadPart_mut() = -500000;

            let path: UnicodeString = default_pipe_name().as_str().into();
            let mut path_raw = path.as_unicode_string();
            let mut object_attributes = OBJECT_ATTRIBUTES {
                Length: mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
                RootDirectory: null_mut(),
                ObjectName: &mut path_raw,
                Attributes: OBJ_CASE_INSENSITIVE,
                SecurityDescriptor: null_mut(),
                SecurityQualityOfService: null_mut(),
            };

            NtCreateNamedPipeFile(
                &mut handle,
                GENERIC_READ | GENERIC_WRITE | SYNCHRONIZE,
                &mut object_attributes,
                &mut io_status_block,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                FILE_OPEN_IF,
                FILE_SYNCHRONOUS_IO_NONALERT,
                FILE_PIPE_MESSAGE_TYPE,
                FILE_PIPE_MESSAGE_MODE,
                // FILE_PIPE_COMPLETE_OPERATION,
                FILE_PIPE_QUEUE_OPERATION,
                16,
                BUFFER_SIZE,
                BUFFER_SIZE,
                &mut timeout,
            ).ok()?;

            Ok(Self(handle))
        }
    }
    
    pub fn listen(&self) -> Result<(), IpcError> {
        let mut status_block: IO_STATUS_BLOCK = unsafe { mem::zeroed() };
        let event = Event::new().map_err(IpcError::CouldNotListen)?;

        let status = NtFsControlFile(
            self.0,
            event.as_handle(),
            None,
            null_mut(),
            &mut status_block,
            FSCTL_PIPE_LISTEN,
            null_mut(),
            0,
            null_mut(),
            0,
        );

        if status.raw() == STATUS_PIPE_LISTENING as i32 {
            return Ok(())
        }

        if status.raw() != 0 && status.raw() != STATUS_PIPE_CONNECTED as i32 {
            return Err(IpcError::CouldNotListen(NtError(status.raw())));
        } 

        Ok(())
    }

    pub fn read(&self) -> Result<DebugVerdict, IpcError> {
        let mut frame = Frame::empty();
        let mut status_block: IO_STATUS_BLOCK = unsafe { mem::zeroed() };

        NtReadFile(
            self.0,
            null_mut(),
            None,
            null_mut(),
            &mut status_block,
            frame.buffer_mut().as_mut_ptr() as _,
            MAX_FRAME_SIZE as u32,
            null_mut(),
            null_mut(),
        )
        .ok()
        .map_err(IpcError::CouldNotRead)?;

        let bytes_read = unsafe { status_block.Information } as usize;

        if bytes_read == 0 {
            return Err(IpcError::Disconnected);
        }

        frame.set_length(bytes_read);

        frame.deserialize().map_err(IpcError::Deserialize)
    }

    pub fn write(&self, event: &DebugEvent) -> Result<(), IpcError> {
        let frame = Frame::serialize(event).map_err(IpcError::Serialize)?;
        let mut status_block: IO_STATUS_BLOCK = unsafe { mem::zeroed() };

        let status = NtWriteFile(
            self.0,
            null_mut(),
            None,
            null_mut(),
            &mut status_block,
            frame.as_ptr() as _,
            frame.len() as u32,
            null_mut(),
            null_mut(),
        );
        
        status.ok()
        .map_err(IpcError::CouldNotWrite)?;

        Ok(())
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        NtClose(self.0);
    }
}