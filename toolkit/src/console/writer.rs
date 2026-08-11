use core::cell::SyncUnsafeCell;
use core::ptr::null_mut;
use winapi::shared::ntdef::{HANDLE, NTSTATUS};
use crate::{Mutex, WriteStrategy, get_peb};
use crate::{println, u16_stack_string::U16CStackString};
use winapi::um::winbase::FILE_TYPE_CHAR;

pub struct ConsoleWriter<S: WriteStrategy> {
    handle: SyncUnsafeCell<HANDLE>,
    buffer: [u16; 1024],
    _strategy: core::marker::PhantomData<S>,
}

unsafe impl<S: WriteStrategy> Sync for ConsoleWriter<S> {}

impl<S: WriteStrategy> ConsoleWriter<S> {
    pub const fn new() -> Self {
        Self {
            handle: SyncUnsafeCell::new(null_mut()),
            buffer: [0; 1024],
            _strategy: core::marker::PhantomData,
        }
    }

    pub fn write(&self, text: &str) -> Result<u32, NTSTATUS> {
        if text.is_empty() {
            return Ok(0);
        }

        let handle = unsafe {


            let handle_ref = self.handle.get();
            if (*handle_ref).is_null() {
                *handle_ref = get_output_handle();
            }
            *handle_ref
        };

        let mut idx = 0;
        let ptr = self.buffer.as_ptr() as *mut u16;
        for ch in text.encode_utf16() {
            if idx < self.buffer.len() {
                unsafe {
                    *ptr.add(idx) = ch;
                }
                idx += 1;
            } else {
                break;
            }
        }
        
        let mut written = 0;
        let status = S::write(
            handle,
            self.buffer.as_ptr() as _,
            idx as u32,
            &mut written,
        );
        
        if status >= 0 {
            Ok(written)
        } else {
            Err(status)
        }
    }

    pub fn writeln(&self, text: &str) -> Result<u32, NTSTATUS> {     
        let written = self.write(text)?;
        let newline_written = self.write("\r\n")?;
        Ok(written + newline_written)
    }
}

pub fn get_output_handle() -> HANDLE {
    unsafe {
        let peb_ptr = get_peb();
        let peb = &*peb_ptr;
        let process_params_ptr = peb.ProcessParameters;
        
        if process_params_ptr.is_null() {
            return core::ptr::null_mut();
        }

        let process_params = &*process_params_ptr;
        
        process_params.StandardOutput
    }
}