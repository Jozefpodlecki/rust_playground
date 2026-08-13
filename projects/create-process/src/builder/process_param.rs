use core::{mem::zeroed, ptr::{self, null_mut}};

use ntapi::ntrtl::RTL_USER_PROCESS_PARAMETERS;
use winapi::{ctypes::c_void, shared::ntdef::HANDLE};

use crate::builder::{error::ProcessBuilderError, types::AlignedBuffer};


pub struct ProcessParamsBuilder<'a, const N: usize> {
    buffer: AlignedBuffer<N>,
    console_handle: HANDLE,
    process_group_id: u32,
    environment_ptr: *mut c_void,
    environment_len: usize,
    params_ptr: *mut RTL_USER_PROCESS_PARAMETERS,
    image_path: &'a str,
    command_args: Option<&'a str>,
    current_directory: Option<&'a str>,
    desktop_info: Option<&'a str>,
    length: usize,
}

impl<'a, const N: usize> ProcessParamsBuilder<'a, N> {
    pub fn new() -> Self {
        assert!(N > size_of::<RTL_USER_PROCESS_PARAMETERS>() + 100);

        Self {
            buffer: AlignedBuffer::new(),
            console_handle: null_mut(),
            process_group_id: 0,
            environment_ptr: null_mut(),
            environment_len: 0,
            params_ptr: null_mut(),
            command_args: None,
            current_directory: None,
            image_path: "",
            desktop_info: Some(r#"Winsta0\Default"#),
            length: 0,
        }
    }
    
    pub fn set_console_handle(&mut self, handle: HANDLE) {
        self.console_handle = handle;
    }

    pub fn set_process_group(&mut self, id: u32) {
        self.process_group_id = id;
    }

    pub fn set_environment(&mut self, environment_ptr: *mut c_void, environment_len: usize) {
        self.environment_ptr = environment_ptr;
        self.environment_len = environment_len;
    }

    // pub fn set_command_args_iter<T, I>(&mut self, args: I) -> Result<(), ProcessBuilderError>
    // where
    //     I: IntoIterator<Item = T>,
    //     T: AsRef<str>,
    // {
    //     let start = self.length;
    //     let ptr = self.buffer.as_mut_ptr() as *mut u16;
        
    //     unsafe {
    //         if self.image_path.contains(' ') {
    //             ptr.add(self.length / 2).write('"' as u16);
    //             self.length += 2;
                
    //             for ch in self.image_path.encode_utf16() {
    //                 if self.length + 2 > N {
    //                     return Err(ProcessBuilderError::BufferOverflow);
    //                 }
    //                 ptr.add(self.length / 2).write(ch);
    //                 self.length += 2;
    //             }
                
    //             ptr.add(self.length / 2).write('"' as u16);
    //             self.length += 2;
    //         } else {
    //             for ch in self.image_path.encode_utf16() {
    //                 if self.length + 2 > N {
    //                     return Err(ProcessBuilderError::BufferOverflow);
    //                 }
    //                 ptr.add(self.length / 2).write(ch);
    //                 self.length += 2;
    //             }
    //         }
            
    //         for arg in args {
    //             let arg_str = arg.as_ref();
    //             ptr.add(self.length / 2).write(' ' as u16);
    //             self.length += 2;
                
    //             let needs_quote = arg_str.contains(' ') || arg_str.is_empty();
    //             if needs_quote {
    //                 ptr.add(self.length / 2).write('"' as u16);
    //                 self.length += 2;
    //             }
                
    //             for ch in arg_str.encode_utf16() {
    //                 if self.length + 2 > N {
    //                     return Err(ProcessBuilderError::BufferOverflow);
    //                 }
    //                 ptr.add(self.length / 2).write(ch);
    //                 self.length += 2;
    //             }
                
    //             if needs_quote {
    //                 ptr.add(self.length / 2).write('"' as u16);
    //                 self.length += 2;
    //             }
    //         }
            
    //         if self.length + 2 > N {
    //             return Err(ProcessBuilderError::BufferOverflow);
    //         }

    //         ptr.add(self.length / 2).write(0);
    //         self.length += 2;
            
    //         let end_offset = N - self.length;
    //         let dest_ptr = self.buffer.as_mut_ptr().add(end_offset) as *mut u16;
            
    //         ptr::copy(ptr, dest_ptr, self.length);
    //         ptr::write_bytes(self.buffer.as_mut_ptr().add(start) as *mut u8, 0, self.length);
            
    //         // self.command_args_ptr = Some((dest_ptr, total_bytes));
            
    //         Ok(())
    //     }
    // }

    pub fn set_command_args(&mut self, command_args: &'a str) {
        self.command_args = Some(command_args);
    }

    pub fn set_current_directory(&mut self, current_directory: &'a str) {
        self.current_directory = Some(current_directory);
    }

    pub fn set_image_path(&mut self, path: &'a str) {
        self.image_path = path;
    }

    pub fn as_mut_ptr(&mut self) -> *mut RTL_USER_PROCESS_PARAMETERS {
        self.params_ptr
    }

    fn align_offset(&self, offset: usize, align: usize) -> usize {
        (offset + align - 1) & !(align - 1)
    }

    fn write_unicode_string(&mut self, s: &str) -> Result<(*mut u16, usize), ProcessBuilderError> {
        self.length = self.align_offset(self.length, 2); 
        let start = self.length;
        let ptr = self.buffer.as_mut_ptr() as *mut u16;

        for ch in s.encode_utf16() {
            if self.length + 2 > N {
                return Err(ProcessBuilderError::BufferOverflow);
            }
            unsafe {
                ptr.add(self.length / 2).write(ch);
            }
            self.length += 2;
        }

        if self.length + 2 > N {
            return Err(ProcessBuilderError::BufferOverflow);
        }

        unsafe {
            ptr.add(self.length / 2).write(0);
        }
        // self.length += 2;

        unsafe { Ok((ptr.add(start / 2), self.length - start)) }
    }

    fn get_directory_from_path(path: &str) -> &str {
        if let Some(last_sep) = path.rfind('\\') {
            &path[..last_sep]
        } else if let Some(last_sep) = path.rfind('/') {
            &path[..last_sep]
        } else {
            path
        }
    }

    pub fn params_mut(&mut self) -> Result<&mut RTL_USER_PROCESS_PARAMETERS, ProcessBuilderError> {
        unsafe { Ok(&mut *(self.params_ptr as *mut RTL_USER_PROCESS_PARAMETERS)) }
    }

    pub fn write_params(&mut self) -> Result<(), ProcessBuilderError> {
        let ptr = self.buffer.as_mut_ptr();
        self.length += size_of::<RTL_USER_PROCESS_PARAMETERS>();
        self.params_ptr = ptr as _;
        Ok(())
    }

    pub fn write_image_path(&mut self) -> Result<(*mut u16, usize), ProcessBuilderError> {
        self.write_unicode_string(self.image_path)
    }

    pub fn write_command_args(&mut self) -> Result<(*mut u16, usize), ProcessBuilderError> {
        self.length = self.align_offset(self.length, 2);
        let start = self.length;
        let ptr = self.buffer.as_mut_ptr() as *mut u16;
        
        unsafe {
            if let Some(cmd_args) = self.command_args {
                ptr.add(self.length / 2).write('"' as u16);
                self.length += 2;
                
                for ch in self.image_path.encode_utf16() {
                    if self.length + 2 > N {
                        return Err(ProcessBuilderError::BufferOverflow);
                    }
                    ptr.add(self.length / 2).write(ch);
                    self.length += 2;
                }
                
                ptr.add(self.length / 2).write('"' as u16);
                self.length += 2;
                
                ptr.add(self.length / 2).write(' ' as u16);
                self.length += 2;
                
                for ch in cmd_args.encode_utf16() {
                    if self.length + 2 > N {
                        return Err(ProcessBuilderError::BufferOverflow);
                    }
                    ptr.add(self.length / 2).write(ch);
                    self.length += 2;
                }
            } else {
                for ch in self.image_path.encode_utf16() {
                    if self.length + 2 > N {
                        return Err(ProcessBuilderError::BufferOverflow);
                    }
                    ptr.add(self.length / 2).write(ch);
                    self.length += 2;
                }
            }
            
            if self.length + 2 > N {
                return Err(ProcessBuilderError::BufferOverflow);
            }

            ptr.add(self.length / 2).write(0);
            // self.length += 2;
            
            Ok((ptr.add(start / 2), self.length - start))
        }
    }

    pub fn write_current_directory(&mut self) -> Result<(*mut u16, usize), ProcessBuilderError> {
        let current_directory = match self.current_directory {
            Some(path) => path,
            None => Self::get_directory_from_path(self.image_path),
        };
        
        self.length = self.align_offset(self.length, 2);
        let start = self.length;
        let ptr = self.buffer.as_mut_ptr() as *mut u16;
        
        unsafe {
            for ch in current_directory.encode_utf16() {
                if self.length + 2 > N {
                    return Err(ProcessBuilderError::BufferOverflow);
                }
                ptr.add(self.length / 2).write(ch);
                self.length += 2;
            }
            
            ptr.add(self.length / 2).write('\\' as u16);
            self.length += 2;
            
            ptr.add(self.length / 2).write(0);
            // self.length += 2;
            
            Ok((ptr.add(start / 2), self.length - start))
        }
    }

    pub fn write_desktop_info(&mut self) -> Result<(*mut u16, usize), ProcessBuilderError> {
        let desktop = self.desktop_info.unwrap_or("");
        self.write_unicode_string(desktop)
    }

    pub fn write_window_title(&mut self) -> Result<(*mut u16, usize), ProcessBuilderError> {
        self.write_unicode_string(self.image_path)
    }

    pub fn offset(&mut self, offset: usize) -> Result<(), ProcessBuilderError> {
        self.length = self.align_offset(self.length, 8);
        if self.length + offset > N {
            return Err(ProcessBuilderError::BufferOverflow);
        }
        self.length += offset;
        Ok(())
    }

    pub fn build(&mut self) -> Result<(), ProcessBuilderError> {

        let environment_ptr = self.environment_ptr;
        let environment_len = self.environment_len;
        let process_group_id = self.process_group_id;
        let console_handle = self.console_handle;

        self.write_params()?;
        self.offset(0x1)?;

        let (ptr, size) = self.write_current_directory()?;
        let params_mut = self.params_mut()?;
        params_mut.CurrentDirectory.DosPath.Buffer = ptr;
        params_mut.CurrentDirectory.DosPath.Length = size as u16;
        params_mut.CurrentDirectory.DosPath.MaximumLength = size as u16 + 2;
        params_mut.Length += size as u32;

        let (ptr, size) = self.write_image_path()?;
        let params_mut = self.params_mut()?;
        params_mut.ImagePathName.Buffer = ptr;
        params_mut.ImagePathName.Length = size as u16;
        params_mut.ImagePathName.MaximumLength = size as u16 + 2;
        params_mut.Length += size as u32;

        let (ptr, size) = self.write_command_args()?;
        let params_mut = self.params_mut()?;
        params_mut.CommandLine.Buffer = ptr;
        params_mut.CommandLine.Length = size as u16;
        params_mut.CommandLine.MaximumLength = size as u16 + 2;
        params_mut.Length += size as u32;

        let (ptr, size) = self.write_desktop_info()?;
        let params_mut = self.params_mut()?;
        params_mut.DesktopInfo.Buffer = ptr;
        params_mut.DesktopInfo.Length = size as u16;
        params_mut.DesktopInfo.MaximumLength = size as u16 + 2;
        params_mut.Length += size as u32;

        let (ptr, size) = self.write_window_title()?;
        let params_mut = self.params_mut()?;
        params_mut.WindowTitle.Buffer = ptr;
        params_mut.WindowTitle.Length = size as u16;
        params_mut.WindowTitle.MaximumLength = size as u16 + 2;
        params_mut.Length += size as u32;

        params_mut.Environment = environment_ptr;
        params_mut.EnvironmentSize = environment_len;
        params_mut.ProcessGroupId = process_group_id;
        params_mut.Flags = 0x1; // PPF_NORMALIZED 
        params_mut.ConsoleHandle = console_handle;
        params_mut.Length += size_of::<RTL_USER_PROCESS_PARAMETERS>() as u32;
        params_mut.MaximumLength += params_mut.Length;

        Ok(())
    }
}