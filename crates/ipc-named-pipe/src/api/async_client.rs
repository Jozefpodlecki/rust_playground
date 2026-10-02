use core::{mem, ptr::null_mut};

use ntapi::ntioapi::{FILE_NON_DIRECTORY_FILE, FILE_OPEN, FILE_SYNCHRONOUS_IO_NONALERT, IO_STATUS_BLOCK};
use win_platform::{NtError, syscalls::*, types::{HANDLE, ObjectAttributes, UnicodeString}};
use winapi::{shared::ntdef::{OBJ_CASE_INSENSITIVE, OBJECT_ATTRIBUTES}, um::winnt::{GENERIC_READ, GENERIC_WRITE, LARGE_INTEGER, SYNCHRONIZE}};

use crate::{error::IpcError, event::Event, types::*, utils::{default_pipe_name, finish}};

pub struct AsyncIpcClient {
    handle: HANDLE,
    read_event: Event,
    write_event: Event,
}

impl AsyncIpcClient {

    pub fn open() -> Result<Self, IpcError> {
        let mut handle = null_mut();
        let mut io_block = unsafe { core::mem::zeroed::<IO_STATUS_BLOCK>() };

        let path: UnicodeString = default_pipe_name().as_str().into();
        let mut obj_attrs = ObjectAttributes::new()
            .name(path)
            .case_insensitive();
        let mut obj_attrs_raw = obj_attrs.as_raw();
 
        NtCreateFile(
            &mut handle,
            GENERIC_READ | GENERIC_WRITE | SYNCHRONIZE,
            &mut obj_attrs_raw,
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