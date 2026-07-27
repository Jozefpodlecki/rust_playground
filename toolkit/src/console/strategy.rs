use core::cell::SyncUnsafeCell;
use core::fmt::{self, Write};
use core::ptr::null_mut;
use core::sync::atomic::{AtomicBool, Ordering};
use ntapi::ntioapi::{IO_STATUS_BLOCK, NtWriteFile};
use winapi::shared::minwindef::{BOOL, DWORD};
use winapi::shared::ntdef::{HANDLE, NTSTATUS, PVOID};
use winapi::shared::ntstatus::STATUS_INVALID_HANDLE;
use winapi::um::consoleapi::{GetConsoleMode, GetConsoleOutputCP, WriteConsoleW};
use winapi::um::fileapi::GetFileType;
use crate::{Mutex, WriteStrategy, get_peb};
use crate::{println, u16_stack_string::U16CStackString};
use winapi::um::winbase::FILE_TYPE_CHAR;

pub const STD_OUTPUT_HANDLE: u32 = 0xFFFFFFF5;
pub const CP_UTF8: u32 = 65001;

pub struct NtWriteStrategy;
pub struct DeviceIoControlStrategy;
pub struct WriteConsoleWStrategy;

impl WriteStrategy for NtWriteStrategy {
    fn write(
        handle: HANDLE,
        buffer: *const u16,
        chars_to_write: u32,
        chars_written: *mut u32,
    ) -> NTSTATUS {
        unsafe {
            let mut io_status_block: IO_STATUS_BLOCK = core::mem::zeroed();
            
            if handle.is_null() || buffer.is_null() || chars_to_write == 0 {
                return STATUS_INVALID_HANDLE;
            }
            
            let bytes_to_write = chars_to_write * 2;
            
            let status = crate::syscalls::NtWriteFile(
                handle,
                core::ptr::null_mut(),
                None,
                core::ptr::null_mut(),
                &mut io_status_block,
                buffer as PVOID,
                bytes_to_write,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            
            if status >= 0 && !chars_written.is_null() {
                let bytes_written = io_status_block.Information as u32;
                *chars_written = bytes_written / 2;
            }
            
            status
        }
    }
}

impl WriteStrategy for DeviceIoControlStrategy {
    fn write(
        handle: HANDLE,
        buffer: *const u16,
        chars_to_write: u32,
        chars_written: *mut u32,
    ) -> NTSTATUS {
        unsafe {
            let mut io_status_block: IO_STATUS_BLOCK = core::mem::zeroed();
            
            if handle.is_null() || buffer.is_null() || chars_to_write == 0 {
                return STATUS_INVALID_HANDLE;
            }
            
            let bytes_to_write = chars_to_write;

            #[repr(C)]
            #[derive(Copy, Clone, Default)]
            pub struct BufferDescriptor {
                pub length: usize,
                pub pointer: *const u8,
            }

            #[repr(C)]
            #[derive(Copy, Clone, Default)]
            pub struct FirstDescriptor {
                pub field1: u32,
                pub field2: u32,
                pub field3: u32,
                pub field4: u32,
            }

            #[repr(C)]
            #[derive(Copy, Clone, Default)]
            pub struct ThirdDescriptor {
                pub field1: u32,
                pub field2: u32,
            }

            #[repr(C)]
            #[derive(Copy, Clone, Default)]
            pub struct ConsoleIoctlInput {
                pub reserved: u64,
                pub count: u32,
                pub flags: u32,
                pub descriptors: [BufferDescriptor; 3],
            }

            let bytes_to_write = chars_to_write as usize * 2;
            let first = FirstDescriptor {
                field1: 0x01000006,
                field2: 0x00000008,
                field3: 0x00000000,
                field4: 0x00000001,
            };
            let third = ThirdDescriptor {
                field1: 0x00000000,
                field2: 0x00000001,
            };
            let mut input = ConsoleIoctlInput::default();
            input.flags = 1;
            input.count = 2;
            input.descriptors = [
                BufferDescriptor {
                    length: core::mem::size_of::<FirstDescriptor>(),
                    pointer: &first as *const FirstDescriptor as *const u8,
                },
                BufferDescriptor {
                    length: bytes_to_write,
                    pointer: buffer as *const u8,
                },
                BufferDescriptor {
                    length: core::mem::size_of::<ThirdDescriptor>(),
                    pointer: &third as *const ThirdDescriptor as *const u8,
                }];
            
            let status = crate::syscalls::NtDeviceIoControlFile(
                handle,
                core::ptr::null_mut(),
                None,
                core::ptr::null_mut(),
                &mut io_status_block,
                0x00500016,
                &input as *const ConsoleIoctlInput as PVOID,
                64,
                core::ptr::null_mut(),
                0,
            );
            
            if status >= 0 && !chars_written.is_null() {
                let bytes_written = io_status_block.Information as u32;
                *chars_written = bytes_written / 2;
            }
            
            status
        }
    }
}

impl WriteStrategy for WriteConsoleWStrategy {
    fn write(
        handle: HANDLE,
        buffer: *const u16,
        chars_to_write: u32,
        chars_written: *mut u32,
    ) -> NTSTATUS {
        unsafe {
            if handle.is_null() || buffer.is_null() || chars_to_write == 0 {
                return STATUS_INVALID_HANDLE;
            }
            
            let result = WriteConsoleW(
                handle,
                buffer as *const winapi::ctypes::c_void,
                chars_to_write,
                chars_written,
                core::ptr::null_mut(),
            );
            
            if result != 0 {
                0
            } else {
                STATUS_INVALID_HANDLE
            }
        }
    }
}

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
            self.buffer.as_ptr(),
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