use core::alloc::{GlobalAlloc, Layout};
use core::ptr;

pub const ARENA_SIZE: usize = 1024 * 1024;
pub const MIN_BLOCK_SIZE: usize = 32;
pub const MAX_ORDER: usize = (ARENA_SIZE / MIN_BLOCK_SIZE).ilog2() as usize;

#[repr(C)]
pub struct BuddyBlock {
    order: u8,
    free: bool,
    next: *mut BuddyBlock,
}

impl BuddyBlock {
    pub const fn size_of() -> usize {
        core::mem::size_of::<Self>()
    }

    pub fn init(self: *mut Self, order: u8) {
        unsafe {
            (*self).order = order;
            (*self).free = true;
            (*self).next = core::ptr::null_mut();
        }
    }

    pub fn order(self: *mut Self) -> u8 {
        unsafe { (*self).order }
    }

    pub fn set_order(self: *mut Self, order: u8) {
        unsafe { (*self).order = order; }
    }

    pub fn free(self: *mut Self) -> bool {
        unsafe { (*self).free }
    }

    pub fn set_free(self: *mut Self, free: bool) {
        unsafe { (*self).free = free; }
    }

    pub fn next(self: *mut Self) -> *mut Self {
        unsafe { (*self).next }
    }

    pub fn set_next(self: *mut Self, next: *mut Self) {
        unsafe { (*self).next = next; }
    }

    pub fn size(self: *mut Self) -> usize {
        MIN_BLOCK_SIZE << unsafe { (*self).order as usize }
    }

    pub fn data(self: *mut Self) -> *mut u8 {
        unsafe { self.cast::<u8>().add(Self::size_of()) }
    }

    pub fn from_ptr(ptr: *mut u8) -> *mut Self {
        unsafe { ptr.sub(Self::size_of()).cast() }
    }

    pub fn add_offset(self: *mut Self, offset: usize) -> *mut Self {
        unsafe { self.cast::<u8>().add(offset).cast() }
    }

    pub fn buddy(self: *mut Self, order: u8) -> *mut Self {
        let block_size = MIN_BLOCK_SIZE << order as usize;
        let block_addr = self as usize;
        
        unsafe {
            if (block_addr / block_size) % 2 == 0 {
                self.add_offset(block_size).cast()
            } else {
                self.add_offset(block_size.wrapping_neg()).cast()
            }
        }
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

pub struct BuddyAllocator<const N: usize>(pub(crate) *mut u8);

unsafe impl<const N: usize> Send for BuddyAllocator<N> {}

impl<const N: usize> BuddyAllocator<N> {
    fn header(&self) -> *mut Header {
        self.0.cast()
    }

    pub(crate) fn first_block(&self) -> *mut BuddyBlock {
        unsafe { self.0.add(Header::size_of()).cast() }
    }

    fn init(&self) {
        self.header().init();
        let first = self.first_block();
        
        for i in 0..=MAX_ORDER {
            self.free_list(i).set_next(core::ptr::null_mut());
        }

        let total_blocks = (N - Header::size_of()) / BuddyBlock::size_of();
        let max_order = (total_blocks).ilog2() as u8;
        first.init(max_order);
        self.add_to_free_list(first);
    }

    pub(crate) fn free_list(&self, order: usize) -> *mut BuddyBlock {
        let base = self.first_block();
        let list_head = base.add_offset(order * BuddyBlock::size_of());
        list_head.cast()
    }

    fn add_to_free_list(&self, block: *mut BuddyBlock) {
        let order = block.order() as usize;
            let head = self.free_list(order);
            block.set_next(head.next());
            head.set_next(block);
            block.set_free(true);
    }

    fn remove_from_free_list(&self, block: *mut BuddyBlock) {
        let order = block.order() as usize;
        let head = self.free_list(order);
        let mut current = head;
        
        while !current.next().is_null() && current.next() != block {
            current = current.next();
        }
        
        if !current.next().is_null() {
            current.set_next(block.next());
            block.set_next(core::ptr::null_mut());
        }
    }

    fn pop_from_free_list(&self, order: usize) -> *mut BuddyBlock {
        let head = self.free_list(order);
        let block = head.next();
        
        if !block.is_null() {
            head.set_next(block.next());
            block.set_next(core::ptr::null_mut());
            block.set_free(false);
        }
        
        block
    }

    fn find_block(&self, order: usize) -> *mut BuddyBlock {
        let block = self.pop_from_free_list(order);
            if !block.is_null() {
                return block;
            }

            for higher_order in (order + 1)..=MAX_ORDER {
                let block = self.pop_from_free_list(higher_order);
                if !block.is_null() {
                    let mut current = block;
                    let mut current_order = higher_order;
                    
                    while current_order > order {
                        current_order -= 1;
                        let half_size = MIN_BLOCK_SIZE << current_order;
                        let buddy = current.add_offset(half_size);
                        
                        buddy.init(current_order as u8);
                        self.add_to_free_list(buddy);
                        current.set_order(current_order as u8);
                    }
                    
                    return current;
                }
            }

            core::ptr::null_mut()
    }

    fn merge_block(&self, block: *mut BuddyBlock) {
        let order = block.order() as usize;
            
        if order >= MAX_ORDER {
            self.add_to_free_list(block);
            return;
        }

        let buddy = block.buddy(order as u8);
        
        if buddy.is_null() || !buddy.free() || buddy.order() != order as u8 {
            self.add_to_free_list(block);
            return;
        }

        self.remove_from_free_list(buddy);
        let parent = if (block as usize) < (buddy as usize) { block } else { buddy };
        parent.set_order((order + 1) as u8);
        self.merge_block(parent);
    }

    pub fn alloc(&self, layout: Layout) -> *mut u8 {
        if !self.header().is_initialized() {
            self.init();
        }

        let size = Self::align_up(layout.size(), layout.align());
        let needed_size = Self::align_up(size + BuddyBlock::size_of(), 8);
        
        if needed_size > N - Header::size_of() {
            return core::ptr::null_mut();
        }

        let order = (needed_size / MIN_BLOCK_SIZE).ilog2() as usize;
        if order > MAX_ORDER {
            return core::ptr::null_mut();
        }

        let block = self.find_block(order);
        if block.is_null() {
            return core::ptr::null_mut();
        }

        block.data()
    }

    pub fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        if ptr.is_null() { return; }
        let block = BuddyBlock::from_ptr(ptr);
        self.merge_block(block);
    }

    fn align_up(size: usize, align: usize) -> usize {
        (size + align - 1) & !(align - 1)
    }

    pub fn free_blocks(&self) -> BuddyBlockIter {
        BuddyBlockIter(self.first_block())
    }
}

unsafe impl<const N: usize> GlobalAlloc for BuddyAllocator<N> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.dealloc(ptr, layout)
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        unsafe {
            let alignment = layout.alignment();
            let new_layout = Layout::from_size_align_unchecked(new_size, alignment.into());
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
            
            let retry_ptr = self.alloc(new_layout);
            if retry_ptr.is_null() {
                return core::ptr::null_mut();
            }

            ptr::copy_nonoverlapping(temp.as_ptr(), retry_ptr, old_size);
            retry_ptr
        }
    }
}

pub struct BuddyBlockIter(*mut BuddyBlock);

#[derive(Debug, Clone, Copy)]
pub struct BuddyBlockInfo {
    pub ptr: *mut BuddyBlock,
    pub order: u8,
    pub size: usize,
    pub free: bool,
}

impl BuddyBlockInfo {
    pub fn ptr(&self) -> *mut BuddyBlock { self.ptr }
    pub fn order(&self) -> u8 { self.order }
    pub fn size(&self) -> usize { self.size }
    pub fn free(&self) -> bool { self.free }
}

impl Iterator for BuddyBlockIter {
    type Item = BuddyBlockInfo;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_null() {
            return None;
        }
        
        let block = self.0;
        let info = BuddyBlockInfo {
            ptr: block,
            order: block.order(),
            size: block.size(),
            free: block.free(),
        };
        
        self.0 = self.0.add_offset(block.size());
        Some(info)
    }
}

impl core::fmt::Display for BuddyBlockInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:p} order={} size={} {}",
            self.ptr,
            self.order,
            self.size,
            if self.free { "free" } else { "used" }
        )
    }
}