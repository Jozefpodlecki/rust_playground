use core::{any::TypeId, mem};

#[derive(Clone, Copy)]
pub struct FuncPtr(pub *const u8);

#[derive(Clone, Copy)]
pub struct DataPtr(pub *mut u8);

#[derive(Clone, Copy)]
pub struct DropFn(pub unsafe fn(*mut u8));

#[derive(Clone, Copy)]
pub struct ClosureMeta {
    pub func_ptr: FuncPtr,
    pub data_ptr: DataPtr,
    pub data_size: usize,
    pub data_align: usize,
    pub type_id: TypeId,
    pub drop_fn: DropFn,
}

#[repr(C)]
pub struct ClosureHeader {
    pub meta: ClosureMeta,
    pub next: *mut ClosureHeader,
}

pub struct BlocksIter {
    current: *mut ClosureHeader,
    data_ptr: *mut u8,
}

impl BlocksIter {
    pub unsafe fn new(current: *mut ClosureHeader, data_ptr: *mut u8) -> Self {
        Self { current, data_ptr }
    }
}

pub struct BlockInfo {
    pub header_offset: usize,
    pub data_offset: usize,
    pub data_size: usize,
    pub data_align: usize,
}

impl Iterator for BlocksIter {
    type Item = BlockInfo;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.current.is_null() {
                return None;
            }
            let header = &*self.current;
            let header_offset = (self.current as usize) - (self.data_ptr as usize);
            let data_offset = header_offset + mem::size_of::<ClosureHeader>();
            let info = BlockInfo {
                header_offset,
                data_offset,
                data_size: header.meta.data_size,
                data_align: header.meta.data_align,
            };
            self.current = header.next;
            Some(info)
        }
    }
}