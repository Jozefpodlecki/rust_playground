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

    fn aligned_offset(&self) -> usize {
        self.aligned_addr() as usize - self.0 as usize
    }

    pub fn first_block(&self) -> *mut BuddyBlock<MIN_BLOCK> {
        self.aligned_addr().cast()
    }

    fn init(&self) {
        self.header().init();
        self.free_list_heads().init();
        let offset = self.aligned_offset();
        let first = self.first_block();
        let total_blocks = (N - offset) / MIN_BLOCK;
        let max_order = (total_blocks).ilog2() as u8;
        first.init(max_order);
        self.add_to_free_list(first);
    }

    fn free_list_heads(&self) -> *mut FreeListHeads<MAX_ORDER, MIN_BLOCK> {
        unsafe { 
            let ptr = self.0.add(Header::size_of()).cast();
            debug_println!("free_list_heads at {:p}", ptr);
            ptr
        }
    }

    pub fn free_list(&self, order: usize) -> *mut BuddyBlock<MIN_BLOCK> {
        let heads = self.free_list_heads();
        unsafe { 
            let ptr = *heads.head(order);
            debug_println!("free_list: order={}, head={:p}", order, ptr);
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
        debug_println!("add_to_free_list: block={:p}, order={}", block, order);
        
        let head = self.free_list(order);
        debug_println!("  current head={:p}", head);
        
        block.set_next(head);
        self.set_free_list(order, block);
        block.set_free(true);
        debug_println!("  added block to free list");
    }

    fn set_free_list(&self, order: usize, block: *mut BuddyBlock<MIN_BLOCK>) {
        unsafe {
            let heads = self.free_list_heads();
            debug_println!("set_free_list: order={}, block={:p}", order, block);
            *heads.head(order) = block;
        }
    }

    fn remove_from_free_list(&self, block: *mut BuddyBlock<MIN_BLOCK>) {
        let order = block.order() as usize;
        debug_println!("remove_from_free_list: block={:p}, order={}", block, order);
        
        let mut current = self.free_list(order);
        debug_println!("  current head={:p}", current);
        
        if current == block {
            debug_println!("  block is head, updating");
            self.set_free_list(order, block.next());
            block.set_next(core::ptr::null_mut());
            return;
        }
        
        while !current.is_null() && current.next() != block {
            current = current.next();
        }
        
        if !current.is_null() && !current.next().is_null() {
            debug_println!("  found block, removing");
            current.set_next(block.next());
            block.set_next(core::ptr::null_mut());
        } else {
            debug_println!("  WARNING: block not found in free list!");
        }
    }

    // fn pop_from_free_list(&self, order: usize) -> *mut BuddyBlock<MIN_BLOCK> {
    //     debug_println!("pop_from_free_list: order={}", order);
    //     let head = self.free_list(order);
    //     debug_println!("  head={:p}", head);
        
    //     if head.is_null() {
    //         debug_println!("  head is null, returning null");
    //         return core::ptr::null_mut();
    //     }
        
    //     // Remove head from free list
    //     let next = head.next();
    //     self.set_free_list(order, next);
    //     head.set_next(core::ptr::null_mut());
    //     head.set_free(false);
    //     debug_println!("  popped successfully, returning {:p}", head);
    //     head
    // }

    fn pop_from_free_list(&self, order: usize) -> *mut BuddyBlock<MIN_BLOCK> {
        debug_println!("pop_from_free_list: order={}", order);
        let head = self.free_list(order);
        debug_println!("  head={:p}", head);

        if head.is_null() {
            debug_println!("  head is null, returning null");
            return core::ptr::null_mut();
        }

        // Validate head is within arena
        let start = self.0;
        let end = unsafe { self.0.add(N) };
        if (head as *mut u8) < start || (head as *mut u8) >= end {
            panic!("Corrupted free list head: {:p} is out of arena", head);
        }

        let next = head.next();
        self.set_free_list(order, next);
        head.set_next(core::ptr::null_mut());
        head.set_free(false);
        debug_println!("  popped successfully, returning {:p}", head);
        head
    }

    fn find_block(&self, order: usize) -> *mut BuddyBlock<MIN_BLOCK> {
        debug_println!("find_block: looking for order {}", order);
        
        let block = self.pop_from_free_list(order);
        if !block.is_null() {
            debug_println!("  found at order {}", order);
            return block;
        }

        for higher_order in (order + 1)..=MAX_ORDER {
            debug_println!("  trying higher order {}", higher_order);
            let block = self.pop_from_free_list(higher_order);
            if !block.is_null() {
                debug_println!("  found at order {}, splitting", higher_order);
                let current = block;
                let mut current_order = higher_order;
                
                while current_order > order {
                    current_order -= 1;
                    let half_size = MIN_BLOCK << current_order;
                    let buddy = current.add_offset(half_size);
                    
                    // Check if buddy is within arena bounds
                    let start = self.0;
                    let end = unsafe { self.0.add(N) };
                    if (buddy as *mut u8) < start || (buddy as *mut u8) >= end {
                        debug_println!("    ERROR: buddy at {:p} is out of bounds! Arena: [{:p}, {:p})", buddy, start, end);
                        self.add_to_free_list(current);
                        return core::ptr::null_mut();
                    }
                    
                    debug_println!("    splitting: creating buddy at {:p} with order {}", buddy, current_order);
                    buddy.init(current_order as u8);
                    self.add_to_free_list(buddy);
                    current.set_order(current_order as u8);
                }
                
                current.set_free(false);
                debug_println!("  returning block at {:p} with order {}", current, order);
                return current;
            }
        }

        debug_println!("  no block found");
        core::ptr::null_mut()
    }

    fn merge_block(&self, block: *mut BuddyBlock<MIN_BLOCK>) {
        let order = block.order() as usize;
        debug_println!("merge_block: block={:p}, order={}", block, order);
            
        if order >= MAX_ORDER {
            debug_println!("  at max order, adding to free list");
            self.add_to_free_list(block);
            return;
        }

        let buddy = block.buddy(order as u8);
        debug_println!("  buddy={:p}", buddy);
        
        if buddy.is_null() {
            debug_println!("  buddy is null");
            self.add_to_free_list(block);
            return;
        }
        
        if !buddy.free() {
            debug_println!("  buddy is not free");
            self.add_to_free_list(block);
            return;
        }
        
        if buddy.order() != order as u8 {
            debug_println!("  buddy order mismatch: {} != {}", buddy.order(), order);
            self.add_to_free_list(block);
            return;
        }

        debug_println!("  merging with buddy");
        self.remove_from_free_list(buddy);
        let parent = if (block as usize) < (buddy as usize) { block } else { buddy };
        parent.set_order((order + 1) as u8);
        debug_println!("  parent={:p}, new order={}", parent, order + 1);
        self.merge_block(parent);
    }

    pub fn alloc(&self, layout: Layout) -> *mut u8 {
        if !self.header().is_initialized() {
            self.init();
        }

        let size = Self::align_up(layout.size(), layout.align());
        let needed_size = Self::align_up(size + BuddyBlock::<MIN_BLOCK>::size_of(), 8);
        let usable = N - self.aligned_offset();
        
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
        debug_println!("dealloc: ptr={:p}", ptr);
        if ptr.is_null() { 
            debug_println!("  ptr is null, returning");
            return; 
        }
        debug_println!("BuddyBlock::<MIN_BLOCK>::from_ptr");
        let block = BuddyBlock::<MIN_BLOCK>::from_ptr(ptr);
        debug_println!("  block={:p}, order={}", block, block.order());
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
        // let alignment = layout.alignment();
        // let new_layout = Layout::from_size_align_unchecked(new_size, alignment.into());
        // let new_ptr = self.alloc(new_layout);
        // if !new_ptr.is_null() {
        //     ptr::copy_nonoverlapping(ptr, new_ptr, core::cmp::min(layout.size(), new_size));
        //     self.dealloc(ptr, layout);
        //     return new_ptr;
        // }
        // core::ptr::null_mut()

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
            // Check that the block is the lower half of its buddy pair (aligned to 2*block_size)
            if (block_addr & ((block_size * 2) - 1)) != 0 {
                return None; // not aligned to merge upward
            }
            let buddy = block.buddy(current_order as u8);
            if buddy.is_null() || !buddy.free() || buddy.order() != current_order as u8 {
                return None;
            }
            // Remove buddy from the free list
            self.remove_from_free_list(buddy);
            // The current block becomes the parent (lower address)
            block.set_order((current_order + 1) as u8);
            // The block remains allocated (its free flag is false)
            current_order += 1;
        }
        Some(current_order)
    }
}