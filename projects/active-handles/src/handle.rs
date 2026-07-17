use core::mem;

use ntapi::ntexapi::{NtQuerySystemInformation, SYSTEM_HANDLE_INFORMATION_EX, SYSTEM_HANDLE_TABLE_ENTRY_INFO_EX, SystemExtendedHandleInformation};
use winapi::shared::ntstatus::STATUS_INFO_LENGTH_MISMATCH;

use crate::error::NtApiError;


const BUFFER_SIZE: usize = 7_500_000;
#[repr(align(8))]
struct AlignedBuffer([u8; BUFFER_SIZE]);
static mut BUFFER: AlignedBuffer = AlignedBuffer([0; BUFFER_SIZE]);


#[derive(Clone, Copy)]
pub struct HandleInfo(SYSTEM_HANDLE_TABLE_ENTRY_INFO_EX);

impl HandleInfo {
    pub fn process_id(&self) -> u32 {
        self.0.UniqueProcessId as u32
    }

    pub fn object(&self) -> usize {
        self.0.Object as _
    }

    pub fn handle_value(&self) -> u64 {
        self.0.HandleValue as u64
    }

    pub fn object_type_index(&self) -> u16 {
        self.0.ObjectTypeIndex as u16
    }

    pub fn handle_attributes(&self) -> u32 {
        self.0.HandleAttributes as u32
    }

    pub fn granted_access(&self) -> u32 {
        self.0.GrantedAccess as u32
    }

    pub fn is_valid(&self) -> bool {
        self.0.HandleValue != 0
    }
}
pub struct SystemHandleIterator {
    index: usize,
    count: usize,
}

impl SystemHandleIterator {
    pub fn new() -> Result<Self, NtApiError> {
        unsafe {
            const STATUS_SUCCESS: i32 = 0;
            let mut return_length = 0;

            let status = NtQuerySystemInformation(
                SystemExtendedHandleInformation,
                BUFFER.0.as_mut_ptr() as *mut _,
                BUFFER_SIZE as u32,
                &mut return_length,
            );

            if status == STATUS_INFO_LENGTH_MISMATCH {
                return Err(NtApiError::BufferTooSmall {
                    needed: return_length as usize,
                    available: BUFFER_SIZE,
                });
            }

            if status != STATUS_SUCCESS {
                return Err(NtApiError::QueryFailed(status));
            }

            let info = BUFFER.0.as_ptr() as *const SYSTEM_HANDLE_INFORMATION_EX;
            let handle_count = (*info).NumberOfHandles as usize;
            let entry_size = mem::size_of::<SYSTEM_HANDLE_TABLE_ENTRY_INFO_EX>();
            let offset = mem::offset_of!(SYSTEM_HANDLE_INFORMATION_EX, Handles);
            
            let total_required_size = offset + (handle_count * entry_size);
            if total_required_size > BUFFER_SIZE {
                return Err(NtApiError::BufferTooSmall {
                    needed: total_required_size,
                    available: BUFFER_SIZE,
                });
            }

            Ok(SystemHandleIterator {
                index: 0,
                count: handle_count,
            })
        }
    }

    pub fn total_count(&self) -> usize {
        self.count
    }
}

impl Iterator for SystemHandleIterator {
    type Item = HandleInfo;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            return None;
        }

        unsafe {
            let offset = mem::offset_of!(SYSTEM_HANDLE_INFORMATION_EX, Handles);
            let entry_size = mem::size_of::<SYSTEM_HANDLE_TABLE_ENTRY_INFO_EX>();
            let current_ptr = BUFFER.0.as_ptr().add(offset + (self.index * entry_size));
            let entry = current_ptr.cast::<SYSTEM_HANDLE_TABLE_ENTRY_INFO_EX>().read_unaligned();
            self.index += 1;
            Some(HandleInfo(entry))
        }
    }
}