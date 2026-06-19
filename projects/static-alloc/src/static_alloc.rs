use core::alloc::{GlobalAlloc, Layout};
use core::cell::{SyncUnsafeCell};
use core::ptr;

const ARENA_SIZE: usize = 1024 * 1024;

#[repr(C)]
pub struct Block {
    size: usize,
    next: *mut Block,
}

#[repr(align(16))]
pub struct Arena {
    data: [u8; ARENA_SIZE],
    head: *mut Block,
}

unsafe impl Sync for Arena {}

impl Arena {
    const fn new() -> Self {
        Self {
            data: [0; ARENA_SIZE],
            head: ptr::null_mut(),
        }
    }

    fn init(&mut self) {
        unsafe {
            let base = self.data.as_ptr() as *mut Block;
            (*base).size = ARENA_SIZE - core::mem::size_of::<Block>();
            (*base).next = ptr::null_mut();
            self.head = base;
        }
    }

    #[inline]
    const fn align_up(x: usize, a: usize) -> usize {
        (x + a - 1) & !(a - 1)
    }

    fn alloc(&mut self, layout: Layout) -> *mut u8 {
        unsafe {
            let size = layout.size();
            let align = layout.align();
            let header_size = core::mem::size_of::<Block>();

            if size == 0 {
                return ptr::null_mut();
            }

            let mut prev: *mut Block = ptr::null_mut();
            let mut cur = self.head;

            while !cur.is_null() {
                let block_start = cur as usize;
                let payload_start = block_start + header_size;
                let payload_aligned = Self::align_up(payload_start, align);
                let padding = payload_aligned - payload_start;
                let total_needed = size + padding;
                let block_payload_size = (*cur).size;

                if total_needed <= block_payload_size {
                    let remaining_payload = block_payload_size - total_needed;
                    let next_block = (*cur).next;

                    if remaining_payload > header_size + 16 {
                        let split_payload_start = block_start + header_size + total_needed;
                        let split_block = split_payload_start as *mut Block;

                        (*split_block).size = remaining_payload - header_size;
                        (*split_block).next = next_block;
                        (*cur).size = total_needed;

                        if prev.is_null() {
                            self.head = split_block;
                        } else {
                            (*prev).next = split_block;
                        }
                    } else {
                        if prev.is_null() {
                            self.head = next_block;
                        } else {
                            (*prev).next = next_block;
                        }
                    }

                    return payload_aligned as *mut u8;
                }

                prev = cur;
                cur = (*cur).next;
            }

            ptr::null_mut()
        }
    }

    fn dealloc(&mut self, ptr: *mut u8, _layout: Layout) {
        unsafe {
            if ptr.is_null() {
                return;
            }

            let header_size = core::mem::size_of::<Block>();
            let block = (ptr as usize - header_size) as *mut Block;

            (*block).next = self.head;
            self.head = block;
        }
    }

    pub fn used(&self) -> usize {
        unsafe {
            let header_size = core::mem::size_of::<Block>();
    let mut total_free = 0;
    let mut block_count = 0;
    let mut cur = self.head;
    
    while !cur.is_null() {
        let block_start = cur as usize;
        let block_size = (*cur).size;
        total_free += header_size + block_size;
        block_count += 1;
        cur = (*cur).next;
    }
    
    // Return the actual used amount
    ARENA_SIZE - total_free
        }
    }
}

pub static mut ARENA: SyncUnsafeCell<Arena> = SyncUnsafeCell::new(Arena::new());

pub struct FreeListAllocator;

unsafe impl GlobalAlloc for FreeListAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let arena = ARENA.get_mut();
        
        if arena.head.is_null() {
            arena.init();
        }
        
        arena.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let arena = ARENA.get_mut();
        
        arena.dealloc(ptr, layout)
    }
}