use core::{mem, ptr::null_mut};

use log::info;
use ntapi::ntioapi::*;
use win_platform::{NtError, NtStatus, syscalls::{NtClose, NtCreateEvent, NtCreateNamedPipeFile, NtFsControlFile, NtReadFile, NtWriteFile}, types::UnicodeString};
use winapi::{shared::{ntdef::{NotificationEvent, OBJ_CASE_INSENSITIVE, OBJECT_ATTRIBUTES}, ntstatus::{STATUS_PIPE_CONNECTED, STATUS_PIPE_LISTENING}}, um::winnt::*};
use win_platform::types::HANDLE;

use crate::{constants::FSCTL_PIPE_LISTEN, error::IpcError, event::Event, types::*, utils::{default_pipe_name, finish}};

const BUFFER_SIZE: u32 = 4096;

pub struct AsyncIpcServer {
    handle: HANDLE,
    read_event: Event,
    write_event: Event,
}

impl AsyncIpcServer {
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
                0,
                FILE_PIPE_MESSAGE_TYPE,
                FILE_PIPE_MESSAGE_MODE,
                FILE_PIPE_QUEUE_OPERATION,
                16,
                BUFFER_SIZE,
                BUFFER_SIZE,
                &mut timeout,
            ).ok()?;

            let read_event = Event::new().map_err(IpcError::CouldNotCreate)?;
            let write_event = Event::new().map_err(IpcError::CouldNotCreate)?;

            Ok(Self { handle, read_event, write_event })
        }
    }
    
    pub fn listen(&self) -> Result<(), IpcError> {
        let mut status_block: IO_STATUS_BLOCK = unsafe { mem::zeroed() };
        let listen_event = Event::new().map_err(IpcError::CouldNotListen)?;

        let status = NtFsControlFile(
            self.handle,
            listen_event.as_handle(),
            None,
            null_mut(),
            &mut status_block,
            FSCTL_PIPE_LISTEN,
            null_mut(),
            0,
            null_mut(),
            0,
        );

        let final_status = finish(status, &listen_event, &status_block)
            .map_err(IpcError::CouldNotListen)?;

        if !final_status.is_success() && final_status.raw() != STATUS_PIPE_CONNECTED as i32 {
            return Err(IpcError::CouldNotListen(NtError(final_status.raw())));
        }

        Ok(())
    }

    pub fn read(&self) -> Result<DebugVerdict, IpcError> {
        let mut frame = Frame::empty();
        let mut status_block: IO_STATUS_BLOCK = unsafe { mem::zeroed() };

        let status = NtReadFile(
            self.handle,
            self.read_event.as_handle(),
            None,
            null_mut(),
            &mut status_block,
            frame.buffer_mut().as_mut_ptr() as _,
            MAX_FRAME_SIZE as u32,
            null_mut(),
            null_mut(),
        );

        let final_status = finish(status, &self.read_event, &status_block)
            .map_err(IpcError::CouldNotRead)?;

        if !final_status.is_success() {
            return Err(IpcError::CouldNotRead(NtError(final_status.raw())));
        }

        let bytes_read = unsafe { status_block.Information } as usize;
        if bytes_read == 0 {
            return Err(IpcError::Disconnected);
        }

        frame.set_length(bytes_read).map_err(IpcError::Deserialize)?;
        frame.deserialize().map_err(IpcError::Deserialize)
    }

    pub fn write(&self, event: &DebugEvent) -> Result<(), IpcError> {
        let frame = Frame::serialize(event).map_err(IpcError::Serialize)?;
        let mut status_block: IO_STATUS_BLOCK = unsafe { mem::zeroed() };

        let status = NtWriteFile(
            self.handle,
            self.write_event.as_handle(),
            None,
            null_mut(),
            &mut status_block,
            frame.as_ptr() as _,
            frame.len() as u32,
            null_mut(),
            null_mut(),
        );
        
        let final_status = finish(status, &self.write_event, &status_block)
            .map_err(IpcError::CouldNotWrite)?;

        if !final_status.is_success() {
            return Err(IpcError::CouldNotWrite(NtError(final_status.raw())));
        }

        Ok(())
    }
}

impl Drop for AsyncIpcServer {
    fn drop(&mut self) {
        NtClose(self.handle);
    }
}