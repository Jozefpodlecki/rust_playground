use core::{alloc::{GlobalAlloc, Layout}, ptr};

#[repr(C)]
pub struct FreeBlock {
    next: *mut FreeBlock,
    size: usize,
}

impl FreeBlock {
    pub fn init_head(self: *mut Self) {
        unsafe {
            (*self).next = core::ptr::null_mut();
            (*self).size = 0;
        }
    }

    pub fn next(self: *mut Self) -> *mut FreeBlock {
        unsafe { (*self).next }
    }

    pub fn set_next(self: *mut Self, next: *mut FreeBlock) {
        unsafe { (*self).next = next; }
    }
    
    pub fn size(self: *mut Self) -> usize {
        unsafe { (*self).size }
    }

    pub const fn size_of() -> usize {
        core::mem::size_of::<Self>()
    }
    
    pub fn set_size(self: *mut Self, size: usize) {
        unsafe { (*self).size = size; }
    }

    pub fn add_offset(self: *mut Self, offset: usize) -> *mut Self {
        unsafe { self.cast::<u8>().add(offset).cast() }
    }
    
    pub fn init_free(self: *mut Self, size: usize) {
        unsafe {
            (*self).next = core::ptr::null_mut();
            (*self).size = size;
        }
    }
    
    pub fn data(self: *mut Self) -> *mut u8 {
        unsafe { self.cast::<u8>().add(Self::size_of()) }
    }
    
    pub fn from_ptr(ptr: *mut u8) -> *mut Self {
        unsafe { ptr.sub(Self::size_of()).cast() }
    }
}

#[repr(align(8))]
struct Header(u8);

impl Header {
    pub const fn size_of() -> usize {
        core::mem::size_of::<Self>()
    }

    pub fn init(self: *mut Self) {
        unsafe { (*self).0 = 1; }
    }
    
    pub fn is_initialized(self: *mut Self) -> bool {
        unsafe { (*self).0 == 1 }
    }
}

pub struct FreeListAllocator<const N: usize>(pub(crate) *mut u8);

impl<const N: usize> FreeListAllocator<N> {

    fn header(&self) -> *mut Header {
        self.0.cast()
    }
    
    fn first_block(&self) -> *mut FreeBlock {
        unsafe {
            self.0.add(Header::size_of()).cast()
        }
    }

    fn init(&self) {
        self.header().init();
        let head = self.first_block();
        head.init_head();
        let free_block = head.add_offset(FreeBlock::size_of());
        let size = N - Header::size_of() - FreeBlock::size_of(); // total bytes from free_block to end
        free_block.init_free(size);
        head.set_next(free_block);
    }


    fn alloc(&self, layout: Layout) -> *mut u8 {
        if !self.header().is_initialized() {
            self.init();
        }
        
        let size = Self::align_up(layout.size(), layout.align());
        let total_size = Self::align_up(size + FreeBlock::size_of(), 8);

        let (block, prev) = match self.find_block(total_size) {
            Some(result) => result,
            None => return core::ptr::null_mut(),
        };
        
        self.remove_from_list(block, prev);
        self.split_block(block, total_size);
        self.coalesce();

        block.data()
    }

    fn find_block(&self, total_size: usize) -> Option<(*mut FreeBlock, *mut FreeBlock)> {
        let mut prev = self.first_block();
        let mut current = prev.next();
        
        while !current.is_null() {
            if current.size() >= total_size {
                return Some((current, prev));
            }
            prev = current;
            current = current.next();
        }
        None
    }

    fn remove_from_list(&self, block: *mut FreeBlock, prev: *mut FreeBlock) {
        prev.set_next(block.next());
    }

    fn align_up(size: usize, align: usize) -> usize {
        (size + align - 1) & !(align - 1)
    }

    fn split_block(&self, block: *mut FreeBlock, total_size: usize) {
        let remaining = block.size() - total_size;
        block.set_size(total_size);

        if remaining > FreeBlock::size_of() {
            let new_block = block.add_offset(total_size);
            new_block.init_free(remaining);

            let head = self.first_block();
            let mut current = head;
            while !current.next().is_null() && (current.next() as usize) < (new_block as usize) {
                current = current.next();
            }
            new_block.set_next(current.next());
            current.set_next(new_block);
        }
    }

    fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        if ptr.is_null() { return; }
        let block = FreeBlock::from_ptr(ptr);
        let head = self.first_block();

        let mut current = head;
        while !current.next().is_null() && (current.next() as usize) < (block as usize) {
            current = current.next();
        }
        
        block.set_next(current.next());
        current.set_next(block);
        self.coalesce();
    }

    pub fn free_blocks(&self) -> FreeBlockIter {
        FreeBlockIter(self.first_block().next())
    }

    fn coalesce(&self) {
        let head = self.first_block();
        let mut current = head.next();
        while !current.is_null() {
            let next = current.next();
            if next.is_null() { break; }
            let current_end = (current as usize) + current.size();
            let next_start = next as usize;
            if current_end == next_start {
                current.set_size(current.size() + next.size());
                current.set_next(next.next());
                continue;
            }
            current = next;
        }
    }
}

unsafe impl<const N: usize> GlobalAlloc for FreeListAllocator<N> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.dealloc(ptr, layout)
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {

        unsafe {
            let alignment = layout.alignment();
            let new_layout = Layout::from_size_alignment_unchecked(new_size, alignment);
            let new_ptr = self.alloc(new_layout);

            if !new_ptr.is_null() {
                ptr::copy_nonoverlapping(ptr, new_ptr, core::cmp::min(layout.size(), new_size));
                self.dealloc(ptr, layout);
                return new_ptr;
            }

            let old_size = layout.size();
            let mut temp = [0u8; N];

            if old_size > temp.len() {
                return core::ptr::null_mut();
            }

            ptr::copy_nonoverlapping(ptr, temp.as_mut_ptr(), old_size);
            self.dealloc(ptr, layout);
            self.coalesce();
            let retry_ptr = self.alloc(new_layout);

            if retry_ptr.is_null() {
                return core::ptr::null_mut();
            }

            ptr::copy_nonoverlapping(temp.as_ptr(), retry_ptr, old_size);
            retry_ptr   
        }
    }
}

pub struct FreeBlockIter(*mut FreeBlock);

#[derive(Debug, Clone, Copy)]
pub struct FreeBlockInfo {
    ptr: *mut FreeBlock,
    size: usize,
}

impl FreeBlockInfo {
    pub fn ptr(&self) -> *mut FreeBlock {
        self.ptr
    }
    
    pub fn size(&self) -> usize {
        self.size
    }
}

impl Iterator for FreeBlockIter {
    type Item = FreeBlockInfo;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_null() {
            None
        } else {
            let block = self.0;
            let size = block.size();
            self.0 = block.next();
            Some(FreeBlockInfo { ptr: block, size })
        }
    }
}

impl core::fmt::Display for FreeBlockInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:p}({})", self.ptr, self.size)
    }
}