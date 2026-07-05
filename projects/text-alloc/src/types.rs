use core::{ptr::null_mut, sync::atomic::{AtomicPtr, Ordering}};

#[repr(C)]
pub struct BlockHeader {
    pub size: usize,
    pub next: AtomicBlockHeader,
}

pub struct AtomicBlockHeader(AtomicPtr<BlockHeader>);

impl Clone for AtomicBlockHeader {
    fn clone(&self) -> Self {
        Self(AtomicPtr::new(self.0.load(Ordering::Acquire)))
    }
}

impl AtomicBlockHeader {
    pub const fn new(value: BlockPtr) -> Self {
        Self(AtomicPtr::new(value.as_ptr()))
    }

    pub fn load(&self) -> BlockPtr {
        BlockPtr(self.0.load(Ordering::Acquire))
    }

    pub fn store(&self, value: BlockPtr) {
        self.0.store(value.as_ptr(), Ordering::Release)
    }

    pub fn cas(&self, current: BlockPtr, new: BlockPtr) -> bool {
        self.0
            .compare_exchange(current.as_ptr(), new.as_ptr(), Ordering::Release, Ordering::Acquire)
            .is_ok()
    }
}

impl Default for AtomicBlockHeader {
    fn default() -> Self {
        Self::new(BlockPtr::null())
    }
}


#[repr(transparent)]
pub struct ArenaPtr(pub *mut u8);

impl ArenaPtr {
    pub unsafe fn as_block_header(self) -> BlockPtr {
        BlockPtr(self.0 as *mut BlockHeader)
    }

    pub unsafe fn as_atomic_block_header(&self) -> *mut AtomicBlockHeader {
        self.0 as *mut AtomicBlockHeader
    }

    pub unsafe fn add(self, offset: usize) -> Self {
        Self(self.0.add(offset))
    }

    pub unsafe fn as_ptr(&self) -> *mut u8 {
        self.0
    }
}

#[repr(transparent)]
pub struct AllocPtr(pub *mut u8);

impl AllocPtr {
    pub unsafe fn as_ptr(self) -> *mut u8 {
        self.0
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct BlockPtr(pub *mut BlockHeader);

impl BlockPtr {
    pub const fn null() -> Self {
        Self(null_mut())
    }

    pub fn is_null(self) -> bool {
        self.0.is_null()
    }

    pub const fn as_ptr(self) -> *mut BlockHeader {
        self.0
    }

    pub unsafe fn size(&self) -> usize {
        (*self.0).size
    }

    pub unsafe fn next(self) -> AtomicBlockHeader {
        (*self.0).next.clone()
    }

    pub unsafe fn as_usize(self) -> usize {
        self.0 as usize
    }

    pub unsafe fn add(self, offset: usize) -> Self {
        Self(self.0.add(offset))
    }

    pub unsafe fn write(self, value: BlockHeader) {
        self.0.write(value)
    }
}