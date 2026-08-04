use core::alloc::Layout;
use core::ptr;

use crate::types::*;

macro_rules! debug_println {
    () => {
        toolkit::print!("\r\n")
    };
    ($($arg:tt)*) => {{
        toolkit::println!($($arg)*);
    }};
}

pub struct BuddyAllocatorImpl<const N: usize, const MIN_BLOCK: usize, const MAX_ORDER: usize>(pub *mut u8);

unsafe impl<const N: usize, const MIN_BLOCK: usize, const MAX_ORDER: usize> Send for BuddyAllocatorImpl<N, MIN_BLOCK, MAX_ORDER> {}

impl<const N: usize, const MIN_BLOCK: usize, const MAX_ORDER: usize> BuddyAllocatorImpl<N, MIN_BLOCK, MAX_ORDER> {
    fn header(&self) -> *mut Header {
        self.0.cast()
    }

    fn aligned_addr(&self) -> *mut u8 {
        let base = self.0 as usize;
        let max_block_size = MIN_BLOCK << MAX_ORDER;
        let aligned = (base + max_block_size - 1) & !(max_block_size - 1);
        aligned as *mut u8
    }

    // pub fn first_block(&self) -> *mut BuddyBlock<MIN_BLOCK> {
    //     let base = self.0 as usize;
    //     let metadata_size = Header::size_of() + FreeListHeads::<MAX_ORDER, MIN_BLOCK>::size_of();
    //     let start = base + metadata_size;
    //     // Align to MIN_BLOCK (not to max_block_size)
    //     let aligned = (start + MIN_BLOCK - 1) & !(MIN_BLOCK - 1);
    //     aligned as *mut BuddyBlock<MIN_BLOCK>
    // }

    pub fn first_block(&self) -> *mut BuddyBlock<MIN_BLOCK> {
    let base = self.0 as usize;
    let metadata_size = Header::size_of() + FreeListHeads::<MAX_ORDER, MIN_BLOCK>::size_of();
    let start = base + metadata_size;
    // Largest order that fits in the usable space (after metadata)
    let max_order = ((N - metadata_size) / MIN_BLOCK).ilog2() as u8;
    let block_size = MIN_BLOCK << max_order;
    let aligned = (start + block_size - 1) & !(block_size - 1);
    aligned as *mut BuddyBlock<MIN_BLOCK>
}

fn init(&self) {
    self.header().init();
    self.free_list_heads().init();
    let first = self.first_block();
    let offset = (first as usize) - (self.0 as usize);
    let total_blocks = (N - offset) / MIN_BLOCK;
    let max_order = (total_blocks).ilog2() as u8;
    first.init(max_order);
    self.add_to_free_list(first);
}


    // fn init(&self) {
    //     self.header().init();
    //     self.free_list_heads().init();
    //     let first = self.first_block();
    //     let offset = (first as usize) - (self.0 as usize);
    //     let total_blocks = (N - offset) / MIN_BLOCK;
    //     let max_order = (total_blocks).ilog2() as u8; // safe because total_blocks > 0
    //     first.init(max_order);
    //     self.add_to_free_list(first);
    // }

    // fn init(&self) {
    //     self.header().init();
    //     self.free_list_heads().init();
    //     let offset = self.aligned_offset();
    //     let first = self.first_block();
    //     let total_blocks = (N - offset) / MIN_BLOCK;
    //     let max_order = (total_blocks).ilog2() as u8;
    //     first.init(max_order);
    //     self.add_to_free_list(first);
    // }

    fn free_list_heads(&self) -> *mut FreeListHeads<MAX_ORDER, MIN_BLOCK> {
        unsafe { 
            let ptr = self.0.add(Header::size_of()).cast();
            ptr
        }
    }

    pub fn free_list(&self, order: usize) -> *mut BuddyBlock<MIN_BLOCK> {
        let heads = self.free_list_heads();
        unsafe { 
            let ptr = *heads.head(order);
            ptr
        }
    }

    fn add_to_free_list(&self, block: *mut BuddyBlock<MIN_BLOCK>) {
         let start = self.0;
        let end = unsafe { self.0.add(N) };
        if (block as *mut u8) < start || (block as *mut u8) >= end {
            panic!("Corrupted block pointer: {:p} is out of arena", block);
        }

        let order = block.order() as usize;
        
        let head = self.free_list(order);
        
        block.set_next(head);
        self.set_free_list(order, block);
        block.set_free(true);
    }

    fn set_free_list(&self, order: usize, block: *mut BuddyBlock<MIN_BLOCK>) {
        unsafe {
            let heads = self.free_list_heads();
            *heads.head(order) = block;
        }
    }

    fn remove_from_free_list(&self, block: *mut BuddyBlock<MIN_BLOCK>) {
        let order = block.order() as usize;
        let mut current = self.free_list(order);
        
        if current == block {
            self.set_free_list(order, block.next());
            block.set_next(core::ptr::null_mut());
            return;
        }
        
        while !current.is_null() && current.next() != block {
            current = current.next();
        }
        
        if !current.is_null() && !current.next().is_null() {
            current.set_next(block.next());
            block.set_next(core::ptr::null_mut());
        }
    }

    fn pop_from_free_list(&self, order: usize) -> *mut BuddyBlock<MIN_BLOCK> {
        let head = self.free_list(order);

        if head.is_null() {
            return core::ptr::null_mut();
        }

        let start = self.0;
        let end = unsafe { self.0.add(N) };
        if (head as *mut u8) < start || (head as *mut u8) >= end {
            panic!("Corrupted free list head: {:p} is out of arena", head);
        }

        let next = head.next();
        self.set_free_list(order, next);
        head.set_next(core::ptr::null_mut());
        head.set_free(false);

        head
    }

    fn find_block(&self, order: usize) -> *mut BuddyBlock<MIN_BLOCK> {
        
        let block = self.pop_from_free_list(order);
        if !block.is_null() {
            return block;
        }

        for higher_order in (order + 1)..=MAX_ORDER {
            let block = self.pop_from_free_list(higher_order);

            if !block.is_null() {
                let current = block;
                let mut current_order = higher_order;
                
                while current_order > order {
                    current_order -= 1;
                    let half_size = MIN_BLOCK << current_order;
                    let buddy = current.add_offset(half_size);
                    
                    
                    let start = self.0;
                    let end = unsafe { self.0.add(N) };

                    if (buddy as *mut u8) < start || (buddy as *mut u8) >= end {
                        self.add_to_free_list(current);
                        return core::ptr::null_mut();
                    }
                    
                    buddy.init(current_order as u8);
                    self.add_to_free_list(buddy);
                    current.set_order(current_order as u8);
                }
                
                current.set_free(false);
                return current;
            }
        }

        core::ptr::null_mut()
    }

    fn merge_block(&self, block: *mut BuddyBlock<MIN_BLOCK>) {
        let order = block.order() as usize;
            
        if order >= MAX_ORDER {
            self.add_to_free_list(block);
            return;
        }

        let buddy = block.buddy(order as u8);
        
        if buddy.is_null() {
            self.add_to_free_list(block);
            return;
        }
        
        if !buddy.free() {
            self.add_to_free_list(block);
            return;
        }
        
        if buddy.order() != order as u8 {
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
        let needed_size = Self::align_up(size + BuddyBlock::<MIN_BLOCK>::size_of(), 8);
        let first = self.first_block();
        let offset = (first as usize) - (self.0 as usize);
        let usable = N - offset;

        if needed_size > usable {
            return core::ptr::null_mut();
        }

        let ratio = (needed_size + MIN_BLOCK - 1) / MIN_BLOCK;
        let mut order = (ratio).ilog2() as usize;
        if (1 << order) < ratio { order += 1; }

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
        if ptr.is_null() { 
            return; 
        }

        let block = BuddyBlock::<MIN_BLOCK>::from_ptr(ptr);
        self.merge_block(block);
    }

    fn align_up(size: usize, align: usize) -> usize {
        (size + align - 1) & !(align - 1)
    }

    pub fn free_blocks(&self) -> BuddyBlockIter<MIN_BLOCK> {
        BuddyBlockIter(self.first_block())
    }
}

impl<const N: usize, const MIN_BLOCK: usize, const MAX_ORDER: usize> BuddyAllocatorImpl<N, MIN_BLOCK, MAX_ORDER> {

    pub fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        self.realloc_inplace(ptr, layout, new_size)
    }

    pub fn realloc_inplace(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let block = BuddyBlock::<MIN_BLOCK>::from_ptr(ptr);
        let current_order = block.order() as usize;

        let needed_size = Self::align_up(new_size + BuddyBlock::<MIN_BLOCK>::size_of(), 8);
        let needed_order = if needed_size == 0 {
            0
        } else {
            ((needed_size + MIN_BLOCK - 1) / MIN_BLOCK).ilog2() as usize
        };

        if needed_order <= current_order {
            return ptr;
        }

        if let Some(_new_order) = self.try_expand_block(block, needed_order) {
            return ptr;
        }

        let alignment = layout.alignment();
        let new_layout = unsafe { Layout::from_size_align_unchecked(new_size, alignment.into()) };
        let new_ptr = self.alloc(new_layout);
        if new_ptr.is_null() {
            return core::ptr::null_mut();
        }
        if new_ptr == ptr {
            return ptr;
        }
        unsafe {
            ptr::copy(ptr, new_ptr, core::cmp::min(layout.size(), new_size));
            self.dealloc(ptr, layout);
        }
        new_ptr
    }

    fn try_expand_block(&self, block: *mut BuddyBlock<MIN_BLOCK>, target_order: usize) -> Option<usize> {
        let mut current_order = block.order() as usize;

        while current_order < target_order {
            let block_size = MIN_BLOCK << current_order;
            let block_addr = block as usize;

            if (block_addr & ((block_size * 2) - 1)) != 0 {
                return None;
            }

            let buddy = block.buddy(current_order as u8);
            if buddy.is_null() || !buddy.free() || buddy.order() != current_order as u8 {
                return None;
            }

            self.remove_from_free_list(buddy);
            block.set_order((current_order + 1) as u8);

            current_order += 1;
        }
        Some(current_order)
    }
}