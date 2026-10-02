use core::{mem, ptr::null_mut};

use ntapi::ntioapi::{FILE_NON_DIRECTORY_FILE, FILE_OPEN, FILE_SYNCHRONOUS_IO_NONALERT, IO_STATUS_BLOCK};
use win_platform::{NtError, syscalls::*, types::{HANDLE, UnicodeString}};
use winapi::{shared::ntdef::{OBJ_CASE_INSENSITIVE, OBJECT_ATTRIBUTES}, um::winnt::{GENERIC_READ, GENERIC_WRITE, LARGE_INTEGER, SYNCHRONIZE}};

use crate::{error::IpcError, event::Event, types::*, utils::{default_pipe_name, finish}};

pub struct AsyncIpcClient {
    handle: HANDLE,
    read_event: Event,
    write_event: Event,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct FILE_PIPE_WAIT_FOR_BUFFER {
    pub Timeout: LARGE_INTEGER,
    pub NameLength: u32,
    pub TimeoutSpecified: u8,
    pub Padding: u8,
    pub Name: [u16; 1],
}

impl AsyncIpcClient {

    pub fn open() -> Result<Self, IpcError> {
        let mut handle = null_mut();
        let mut io_block = unsafe { core::mem::zeroed::<IO_STATUS_BLOCK>() };

        let path: UnicodeString = default_pipe_name().as_str().into();
        let mut path_raw = path.as_unicode_string();
        let mut obj_attr = OBJECT_ATTRIBUTES {
            Length: mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
            RootDirectory: null_mut(),
            ObjectName: &mut path_raw,
            Attributes: OBJ_CASE_INSENSITIVE,
            SecurityDescriptor: null_mut(),
            SecurityQualityOfService: null_mut(),
        };

        NtCreateFile(
            &mut handle,
            GENERIC_READ | GENERIC_WRITE | SYNCHRONIZE,
            &mut obj_attr,
            &mut io_block,
            null_mut(),
            0,
            0,
            FILE_OPEN,
            FILE_SYNCHRONOUS_IO_NONALERT | FILE_NON_DIRECTORY_FILE,
            null_mut(),
            0,
        ).ok()?;

        let read_event = Event::new().map_err(IpcError::CouldNotConnect)?;
        let write_event = Event::new().map_err(IpcError::CouldNotConnect)?;

        Ok(Self { handle, read_event, write_event })
    }

    pub fn read(&self) -> Result<DebugEvent, IpcError> {
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

    pub fn write(&self, event: &DebugVerdict) -> Result<(), IpcError> {
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