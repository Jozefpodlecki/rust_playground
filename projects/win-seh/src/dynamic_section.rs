use alloc::boxed::Box;
use ntapi::ntmmapi::{NtAllocateVirtualMemory, NtFreeVirtualMemory};
use winapi::{ctypes, shared::ntdef::{HANDLE, NT_SUCCESS}, um::winnt::{MEM_COMMIT, PAGE_EXECUTE_READWRITE, RUNTIME_FUNCTION, RtlAddFunctionTable, RtlDeleteFunctionTable, RtlLookupFunctionEntry}};

use crate::{code_buffer::CodeBuffer, code_writer::{CodeWriter, FaultingCode}, types::*};

pub struct ExceptionData {
    pub ret_addr: u64
}

#[derive(Debug, Clone, Copy)]
pub enum SetupError {
    NtAllocateVirtualMemory,
    NtWriteVirtualMemory,
    RtlAddFunctionTable,
    RtlLookupFunctionEntry
}

#[repr(C)]
pub struct DynamicSection {
    pub code: CodeBuffer<0x600>,
    pub unwind_info_bytes: [u8; 8],
    pub exception_data: ExceptionData,
    pub trampoline_offset: usize,
    pub function_table: *mut RUNTIME_FUNCTION
}

impl DynamicSection {
    pub fn new() -> Result<*mut Self, SetupError> {
        unsafe {
            let mut base_ptr: *mut winapi::ctypes::c_void = core::ptr::null_mut();
            let mut size = core::mem::size_of::<Self>();
            let handle = -1isize as HANDLE;

            let status = NtAllocateVirtualMemory(
                handle,
                &mut base_ptr,
                0,
                &mut size,
                MEM_COMMIT,
                PAGE_EXECUTE_READWRITE,
            );

            if !NT_SUCCESS(status) {
                return Err(SetupError::NtAllocateVirtualMemory);
            }

            let base = base_ptr as *mut Self;
            (*base).code = CodeBuffer::new(base_ptr as u64);
            Ok(base)
        }
    }

    pub fn setup_function_table(begin: u32, end: u32, unwind_info_rva: u32) -> RUNTIME_FUNCTION {
        let mut func = RUNTIME_FUNCTION {
            BeginAddress: begin,
            EndAddress: end,
            u: unsafe { core::mem::zeroed() },
        };
        unsafe {
            *func.u.UnwindInfoAddress_mut() = unwind_info_rva;
        }
        func
    }

    pub fn write_trampoline(self: *mut Self, handler: *const ()) -> usize {
        unsafe {
            let section = &mut *self;
            let code = &mut section.code;

            let trampoline_offset = code.len();
            section.trampoline_offset = trampoline_offset;
            code.push_bytes(&[0x0f, 0x1f, 0x00]);
            code.push_bytes(&[0x48, 0xb8]); // mov rax, handler
            code.push_u64(handler as u64);
            code.push_bytes(&[0xff, 0xe0]); // jmp rax

            trampoline_offset
        }
    }

    pub fn write_code(self: *mut Self, code_writer: &dyn CodeWriter, handler: *const () ) {
        unsafe {
            let section = &mut *self;
            let code = &mut section.code;

            code.push_bytes(code_writer.write());

            let trampoline_start = self.write_trampoline(handler);
            let handler_rva = self.handler_rva();

            let mut unwind_info = UnwindInfo::new();
            unwind_info.version_flags = VersionFlags::EHANDLER;
            unwind_info.set_exception_handler(handler_rva);
            section.unwind_info_bytes = unwind_info.build();
        }
    }

    pub fn section_base(self: *const Self) -> u64 {
        self as *const _ as u64
    }

    pub fn code_rva(self: *const Self) -> u32 {
        unsafe {
            let section = &*self;
            let code = &section.code;
            code.rva()
        }
    }

    pub fn handler_rva(self: *const Self) -> u32 {
        unsafe {
            let section = &*self;
            let code = &section.code;
            let code_rva = code.rva();

            code.rva_at(section.trampoline_offset)
        }
    }

    pub fn unwind_info_rva(self: *const Self) -> u32 {
        unsafe {
            let section = &*self;
            let code = &section.code;
            let section_base = code.base;
            (&section.unwind_info_bytes as *const _ as u64 - section_base) as u32
        }
    }

    pub fn check(self: *const Self) -> Result<(), SetupError> {
        unsafe {
            let mut image_base: u64 = 0;
            let entry = RtlLookupFunctionEntry(
                (*self).code.as_ptr() as u64,
                &mut image_base,
                core::ptr::null_mut(),
            );
            
            if entry.is_null() {
                Err(SetupError::RtlLookupFunctionEntry)
            } else {
                Ok(())
            }
        }
    }

    pub fn register(self: *mut Self) -> Result<(), SetupError> {
        unsafe {
            let code_rva = self.code_rva();
            let handler_rva = self.handler_rva();
            let unwind_info_rva = self.unwind_info_rva();
            let mut function_table = Box::leak(Box::new(Self::setup_function_table(code_rva, handler_rva, unwind_info_rva)));
            let status = RtlAddFunctionTable(
                &mut *function_table,
                1,
                self.section_base(),
            );
            (*self).function_table = function_table;

            if status == 0 {
                Err(SetupError::RtlAddFunctionTable)
            } else {
                Ok(())
            }
        }
    }

    pub fn call(self: *mut Self) -> u64 {
        unsafe {
            let call_fn: extern "C" fn() -> u64 = core::mem::transmute((*self).code.as_ptr());
            call_fn()
        }
    }
}

impl Drop for DynamicSection {
    fn drop(&mut self) {
        unsafe {
            if !self.function_table.is_null() {
                RtlDeleteFunctionTable(self.function_table);
            }
            let mut base_ptr = self as *mut Self as *mut winapi::ctypes::c_void;
            let mut size = core::mem::size_of::<Self>();
            NtFreeVirtualMemory(
                -1isize as HANDLE,
                &mut base_ptr,
                &mut size,
                0x8000, // MEM_RELEASE
            );
        }
    }
}