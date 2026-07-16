// src/heap.rs
use core::ptr;
use winapi::{
    ctypes::c_void, shared::ntdef::{BOOLEAN, NTSTATUS, PVOID, ULONG, USHORT}, um::winnt::{HEAP_INFORMATION_CLASS, HeapCompatibilityInformation, HeapEnableTerminationOnCorruption},
};
use ntapi::ntrtl::*;

use crate::{error::{HeapError, HeapResult}, flags::HeapFlags};

#[derive(Clone, Copy, Default)]
pub struct HeapParameters {
    pub flags: HeapFlags,
    pub base: *mut c_void,
    pub reserve_size: usize,
    pub commit_size: usize,
    pub lock: *mut c_void,
    pub parameters: PRTL_HEAP_PARAMETERS,
}

#[derive(Clone)]
pub struct NtHeap(*mut c_void);

impl NtHeap {
    pub fn new(args: HeapParameters) -> HeapResult<Self> {
        let handle = unsafe {
            RtlCreateHeap(
                args.flags.bits(),
                args.base,
                args.reserve_size,
                args.commit_size,
                args.lock,
                args.parameters,
            )
        };
        
        if handle.is_null() {
            Err(HeapError(0))
        } else {
            Ok(Self(handle))
        }
    }

    pub fn from_peb() -> Self {
        let peb = toolkit::ProcessEnvironmentBlock::current_process();
        Self(peb.process_heap() as *mut winapi::ctypes::c_void)
    }

    pub fn handle(&self) -> *mut c_void {
        self.0
    }

    pub fn allocate(&self, size: usize, flags: HeapFlags) -> HeapResult<*mut u8> {
        let ptr = unsafe {
            RtlAllocateHeap(
                self.0,
                flags.bits(),
                size,
            )
        };
        
        if ptr.is_null() {
            Err(HeapError(0))
        } else {
            Ok(ptr as *mut u8)
        }
    }

    pub fn allocate_zeroed(&self, size: usize) -> HeapResult<*mut u8> {
        self.allocate(size, HeapFlags::ZERO_MEMORY)
    }

    pub fn reallocate(&self, ptr: *mut u8, size: usize, flags: HeapFlags) -> HeapResult<*mut u8> {
        let new_ptr = unsafe {
            RtlReAllocateHeap(
                self.0,
                flags.bits(),
                ptr as *mut c_void,
                size,
            )
        };
        
        if new_ptr.is_null() {
            Err(HeapError(0))
        } else {
            Ok(new_ptr as *mut u8)
        }
    }

    pub fn free(&self, ptr: *mut u8, flags: HeapFlags) -> bool {
        unsafe {
            RtlFreeHeap(
                self.0,
                flags.bits(),
                ptr as *mut c_void,
            ) != 0
        }
    }

    pub fn size(&self, ptr: *mut u8, flags: HeapFlags) -> usize {
        unsafe {
            RtlSizeHeap(
                self.0,
                flags.bits(),
                ptr as *mut c_void,
            )
        }
    }

    pub fn zero(&self, flags: HeapFlags) -> HeapResult<()> {
        let status = unsafe { RtlZeroHeap(self.0, flags.bits()) };
        if status == 0 {
            Ok(())
        } else {
            Err(HeapError(status))
        }
    }

    pub fn protect(&self, read_only: bool) {
        unsafe { RtlProtectHeap(self.0, read_only as BOOLEAN) }
    }

    pub fn lock(&self) -> bool {
        unsafe { RtlLockHeap(self.0) != 0 }
    }

    pub fn unlock(&self) -> bool {
        unsafe { RtlUnlockHeap(self.0) != 0 }
    }

    pub fn compact(&self, flags: HeapFlags) -> usize {
        unsafe { RtlCompactHeap(self.0, flags.bits()) }
    }

    pub fn validate(&self, flags: HeapFlags, ptr: *mut u8) -> bool {
        unsafe { RtlValidateHeap(self.0, flags.bits(), ptr as *mut c_void) != 0 }
    }

    pub fn validate_all() -> bool {
        unsafe { RtlValidateProcessHeaps() != 0 }
    }

    pub fn is_lfh(&self) -> bool {
        let mut info: ULONG = 0;
        let status = unsafe {
            RtlQueryHeapInformation(
                self.0,
                HeapCompatibilityInformation,
                &mut info as *mut _ as PVOID,
                core::mem::size_of::<ULONG>(),
                ptr::null_mut(),
            )
        };
        status == 0 && info == 2
    }

    pub fn query_information(&self, info_class: HEAP_INFORMATION_CLASS) -> Result<ULONG, NTSTATUS> {
        let mut info: ULONG = 0;
        let status = unsafe {
            RtlQueryHeapInformation(
                self.0,
                info_class,
                &mut info as *mut _ as PVOID,
                core::mem::size_of::<ULONG>(),
                ptr::null_mut(),
            )
        };
        if status == 0 {
            Ok(info)
        } else {
            Err(status)
        }
    }

    pub fn set_terminate_on_corruption(&self, enable: bool) -> bool {
        let value: ULONG = if enable { 1 } else { 0 };
        unsafe {
            RtlSetHeapInformation(
                self.0,
                HeapEnableTerminationOnCorruption,
                &value as *const _ as PVOID,
                core::mem::size_of::<ULONG>(),
            ) == 0
        }
    }

    pub fn process_heaps<const MAX: usize>() -> ProcessHeapsIter<MAX> {
        ProcessHeapsIter::<MAX>::new()
    }

    pub fn walk(&self, entry: &mut RTL_HEAP_WALK_ENTRY) -> HeapResult<()> {
        let status = unsafe { RtlWalkHeap(self.0, entry) };
        if status == 0 {
            Ok(())
        } else {
            Err(HeapError(status))
        }
    }

    pub fn destroy(self) -> bool {
        unsafe { !RtlDestroyHeap(self.0).is_null() }
    }

    pub fn multiple_alloc<const N: usize>(
        &self,
        size: usize,
        flags: HeapFlags,
    ) -> HeapResult<heapless::Vec<*mut u8, N>> {
        let mut array: heapless::Vec<*mut u8, N> = heapless::Vec::new();
        array.resize_default(N).map_err(|_| HeapError(0))?;
        
        let allocated = unsafe {
            RtlMultipleAllocateHeap(
                self.0,
                flags.bits(),
                size,
                N as ULONG,
                array.as_mut_ptr() as *mut _,
            )
        };
        
        if allocated == N as ULONG {
            Ok(array)
        } else {
            array.truncate(allocated as usize);
            Err(HeapError(0))
        }
    }

    pub fn multiple_free(&self, ptrs: &mut [*mut u8], flags: HeapFlags) -> bool {
        unsafe {
            RtlMultipleFreeHeap(
                self.0,
                flags.bits(),
                ptrs.len() as ULONG,
                ptrs.as_mut_ptr() as *mut *mut c_void,
            ) != 0
        }
    }

    pub fn set_user_value(&self, ptr: *mut u8, value: *mut c_void) -> bool {
        unsafe {
            RtlSetUserValueHeap(
                self.0,
                0,
                ptr as *mut c_void,
                value,
            ) != 0
        }
    }

    pub fn get_user_value(&self, ptr: *mut u8) -> Option<*mut c_void> {
        let mut value = ptr::null_mut();
        let result = unsafe {
            RtlGetUserInfoHeap(
                self.0,
                0,
                ptr as *mut c_void,
                &mut value,
                ptr::null_mut(),
            )
        };
        
        if result != 0 {
            Some(value)
        } else {
            None
        }
    }

    pub fn set_user_flags(
        &self,
        ptr: *mut u8,
        flags_reset: HeapFlags,
        flags_set: HeapFlags,
    ) -> bool {
        unsafe {
            RtlSetUserFlagsHeap(
                self.0,
                0,
                ptr as *mut c_void,
                flags_reset.bits(),
                flags_set.bits(),
            ) != 0
        }
    }

    pub fn create_tag(
        &self,
        tag_prefix: u16,
        tag_names: u16,
    ) -> Option<u32> {
        unsafe {
            let tag = RtlCreateTagHeap(
                self.0,
                0,
                tag_prefix as _,
                tag_names as _,
            );
            if tag != 0 { Some(tag) } else { None }
        }
    }
}

impl Drop for NtHeap {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { RtlDestroyHeap(self.0); }
        }
    }
}

unsafe impl Send for NtHeap {}
unsafe impl Sync for NtHeap {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_heap() {
        let heap = NtHeap::new(HeapParameters::default()).unwrap();
        assert!(!heap.handle().is_null());
    }

    #[test]
    fn allocate_and_free() {
        let heap = NtHeap::new(HeapParameters::default()).unwrap();
        let ptr = heap.allocate(100, HeapFlags::ZERO_MEMORY).unwrap();
        assert!(!ptr.is_null());
        
        let size = heap.size(ptr, HeapFlags::NONE);
        assert!(size >= 100);
        
        assert!(heap.free(ptr, HeapFlags::NONE));
    }

    #[test]
    fn reallocate() {
        let heap = NtHeap::new(HeapParameters::default()).unwrap();
        let ptr = heap.allocate(100, HeapFlags::NONE).unwrap();
        
        let new_ptr = heap.reallocate(ptr, 200, HeapFlags::NONE).unwrap();
        assert!(!new_ptr.is_null());
        assert!(heap.free(new_ptr, HeapFlags::NONE));
    }

    #[test]
    fn multiple_allocations() {
        let heap = NtHeap::new(HeapParameters::default()).unwrap();
        let ptrs = heap.multiple_alloc(64, HeapFlags::ZERO_MEMORY, 10).unwrap();
        assert_eq!(ptrs.len(), 10);
        
        for ptr in &ptrs {
            assert!(!ptr.is_null());
        }
        
        let mut ptrs_mut = ptrs;
        assert!(heap.multiple_free(&mut ptrs_mut, HeapFlags::NONE));
    }

    #[test]
    fn user_value() {
        let heap = NtHeap::new(HeapParameters::default()).unwrap();
        let ptr = heap.allocate(100, HeapFlags::NONE).unwrap();
        let value = 0x12345678 as *mut c_void;
        
        assert!(heap.set_user_value(ptr, value));
        assert_eq!(heap.get_user_value(ptr), Some(value));
        
        assert!(heap.free(ptr, HeapFlags::NONE));
    }

    #[test]
    fn process_heaps() {
        let heaps = NtHeap::get_process_heaps();
        assert!(!heaps.is_empty());
    }
}

pub struct ProcessHeapsIter<const MAX: usize> {
    count: ULONG,
    current: ULONG,
    heaps: [*mut c_void; MAX],
}

impl<const MAX: usize> ProcessHeapsIter<MAX> {
    pub fn new() -> Self {
        let mut iter = Self {
            count: 0,
            current: 0,
            heaps: [ptr::null_mut(); MAX],
        };
        
        let count = unsafe { RtlGetProcessHeaps(0, ptr::null_mut()) };
        if count > 0 && (count as usize) <= iter.heaps.len() {
            let actual = unsafe { RtlGetProcessHeaps(count, iter.heaps.as_mut_ptr()) };
            iter.count = actual;
        }
        
        iter
    }
}

impl<const MAX: usize> Iterator for ProcessHeapsIter<MAX> {
    type Item = NtHeap;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.count {
            None
        } else {
            let heap = self.heaps[self.current as usize];
            self.current += 1;
            Some(NtHeap(heap))
        }
    }
}