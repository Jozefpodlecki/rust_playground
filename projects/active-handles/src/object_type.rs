use core::{cell::SyncUnsafeCell, fmt, mem, ops::Deref, slice, str};

use ntapi::ntobapi::{NtQueryObject, OBJECT_TYPE_INFORMATION, ObjectTypesInformation};
use toolkit::{print, println};
use winapi::shared::ntstatus::STATUS_INFO_LENGTH_MISMATCH;

use crate::{error::NtApiError, handle::HandleInfo};

#[repr(C)]
pub struct OBJECT_ALL_TYPES_INFORMATION {
    pub NumberOfObjectTypes: u32,
    pub ObjectTypeInformation: [OBJECT_TYPE_INFORMATION; 1],
}

pub const ObjectAllTypesInformation: u32 = 3;

const TYPE_BUFFER_SIZE: usize = 16_384;
static TYPE_BUFFER: SyncUnsafeCell<[u8; TYPE_BUFFER_SIZE]> = SyncUnsafeCell::new([0; TYPE_BUFFER_SIZE]);
static PARSED_BUFFER: SyncUnsafeCell<[u8; TYPE_BUFFER_SIZE]> = SyncUnsafeCell::new([0; TYPE_BUFFER_SIZE]);

pub struct ObjectTypeIterator<'a> {
    current_index: usize,
    total_types: usize,
    current_ptr: *const u8,
    _marker: core::marker::PhantomData<&'a ()>,
}

impl<'a> ObjectTypeIterator<'a> {

    pub fn new() -> Result<Self, NtApiError> {
        unsafe {
            const STATUS_SUCCESS: i32 = 0;
            let mut return_length = 0;

            let buffer_ptr = TYPE_BUFFER.get() as *mut u8;
            let buffer_size = TYPE_BUFFER_SIZE;

            let status = NtQueryObject(
                core::ptr::null_mut(),
                ObjectTypesInformation,
                buffer_ptr as *mut _,
                buffer_size as u32,
                &mut return_length,
            );

            if status == STATUS_INFO_LENGTH_MISMATCH {
                return Err(NtApiError::BufferTooSmall {
                    needed: return_length as usize,
                    available: buffer_size,
                });
            }

            if status != STATUS_SUCCESS {
                return Err(NtApiError::QueryFailed(status));
            }

            let info_ptr = buffer_ptr as *const OBJECT_ALL_TYPES_INFORMATION;
            let total_types = (*info_ptr).NumberOfObjectTypes as usize;
            let offset = mem::offset_of!(OBJECT_ALL_TYPES_INFORMATION, ObjectTypeInformation);
            let mut test_ptr = buffer_ptr.add(offset);

            for i in 0..total_types {
                let type_info = test_ptr as *const OBJECT_TYPE_INFORMATION;
                let type_name = &(*type_info).TypeName;
                let entry_size = mem::size_of::<OBJECT_TYPE_INFORMATION>() + type_name.MaximumLength as usize;
                let align = mem::align_of::<OBJECT_TYPE_INFORMATION>();
                let aligned_size = (entry_size + align - 1) & !(align - 1);

                test_ptr = test_ptr.add(aligned_size);

                let offset_from_start = test_ptr as usize - buffer_ptr as usize;
                if offset_from_start > buffer_size {
                    return Err(NtApiError::BufferTooSmall {
                        needed: offset_from_start,
                        available: buffer_size,
                    });
                }
            }

            Ok(ObjectTypeIterator {
                current_index: 0,
                total_types,
                current_ptr: buffer_ptr.add(offset),
                _marker: core::marker::PhantomData,
            })
        }
    }
}

pub struct ObjectType {
    pub id: usize,
    str_data: [u8; 64],
    str_len: usize,
}

impl ObjectType {
    pub fn name(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.str_data[..self.str_len]) }
    }
}

impl<'a> Iterator for ObjectTypeIterator<'a> {
    type Item = ObjectType;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.total_types {
            return None;
        }

        unsafe {
            let type_info = self.current_ptr as *const OBJECT_TYPE_INFORMATION;
            let type_name = &(*type_info).TypeName;
            
            let char_count = (type_name.Length / 2) as usize;
            let wide_slice = slice::from_raw_parts(type_name.Buffer, char_count);
            let name = char::decode_utf16(wide_slice.iter().cloned());
            let entry_size = mem::size_of::<OBJECT_TYPE_INFORMATION>() + type_name.MaximumLength as usize;
            let align = mem::align_of::<OBJECT_TYPE_INFORMATION>();
            let aligned_size = (entry_size + align - 1) & !(align - 1);
            self.current_ptr = self.current_ptr.add(aligned_size);

            let mut str_data = [0u8; 64];
            let mut str_len = 0;
            
            for c in name {
                if let Ok(ch) = c {
                    let mut bytes = [0u8; 4];
                    let encoded = ch.encode_utf8(&mut bytes);
                    let bytes_slice = encoded.as_bytes();
                    
                    for &b in bytes_slice {
                        if str_len < 63 {
                            str_data[str_len] = b;
                            str_len += 1;
                        }
                    }
                }
            }

            self.current_index += 1;

            Some(ObjectType { id: self.current_index + 1, str_data, str_len })
        }
    }
}