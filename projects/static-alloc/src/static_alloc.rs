use core::alloc::{GlobalAlloc, Layout};
use core::cell::{SyncUnsafeCell};
use core::ptr::{self};
use core::sync::atomic::{AtomicPtr, Ordering};

const ARENA_SIZE: usize = 1024 * 1024;


#[repr(C, align(16))]
struct Block {
    size: usize,
    next: *mut Block,
}

impl Block {
    #[inline]
    const fn header_size() -> usize {
        core::mem::size_of::<Block>()
    }

    #[inline]
    fn split(&mut self, needed: usize) -> *mut Block {
        unsafe {
            let remaining = self.size - needed - Self::header_size();
            let new_block = (self as *mut Self as usize + Self::header_size() + needed) as *mut Block;
            (*new_block).size = remaining;
            (*new_block).next = self.next;
            self.size = needed;
            self.next = new_block;
            new_block
        }
    }

    #[inline]
    fn coalesce_with_next(&mut self, next: *mut Block) {
        unsafe {
            self.size += (*next).size + Self::header_size();
            self.next = (*next).next;
        }
    }

    #[inline]
    fn can_split(&self, needed: usize) -> bool {
        self.size >= needed + Self::header_size() + 16
    }
}

struct Arena {
    data: [u8; ARENA_SIZE],
    head: AtomicPtr<Block>,
    initialized: bool,
}

unsafe impl Sync for Arena {}

impl Arena {
    const fn new() -> Self {
        Arena {
            data: [0; ARENA_SIZE],
            head: AtomicPtr::new(ptr::null_mut()),
            initialized: false,
        }
    }

    fn init(&mut self) {
        unsafe {
            if !self.initialized {
                let block = self.data.as_mut_ptr() as *mut Block;
                (*block).size = ARENA_SIZE - Block::header_size();
                (*block).next = ptr::null_mut();
                self.head.store(block, Ordering::Release);
                self.initialized = true;
            }
        }
    }

    #[inline]
    fn align_up(x: usize, align: usize) -> usize {
        (x + align - 1) & !(align - 1)
    }

    fn alloc(&mut self, size: usize, align: usize) -> *mut u8 {
        unsafe {
            let mut current = self.head.load(Ordering::Acquire);
            let mut prev: *mut Block = ptr::null_mut();

            while !current.is_null() {
                let block = &mut *current;
                let block_start = current as usize;
                let header_size = Block::header_size();
                let storage_size = core::mem::size_of::<*mut Block>();

                // Minimum address for user data: block_start + header + storage (for the stored pointer)
                let min_user_data = block_start + header_size + storage_size;
                let aligned_user_data = Self::align_up(min_user_data, align);
                let total_needed = (aligned_user_data - block_start) + size;

                if total_needed <= block.size {
                    if block.can_split(total_needed) {
                        let remaining = block.size - total_needed - header_size;
                        let new_block = (block_start + total_needed) as *mut Block;
                        (*new_block).size = remaining;
                        (*new_block).next = block.next;
                        block.size = total_needed;
                        block.next = new_block;
                    }

                    if prev.is_null() {
                        self.head.store(block.next, Ordering::Release);
                    } else {
                        (*prev).next = block.next;
                    }

                    block.next = ptr::null_mut();

                    let user_ptr = aligned_user_data as *mut u8;
                    let storage_ptr = (user_ptr as usize - storage_size) as *mut *mut Block;
                    *storage_ptr = block as *mut Block;

                    return user_ptr;
                }

                prev = current;
                current = block.next;
            }

            ptr::null_mut()
        }
    }

    fn dealloc(&mut self, ptr: *mut u8, _size: usize) {
        unsafe {
            let storage_size = core::mem::size_of::<*mut Block>();
            let storage_ptr = (ptr as usize - storage_size) as *mut *mut Block;
            let block_ptr = *storage_ptr;

            if block_ptr.is_null() {
                return;
            }

            let block = &mut *block_ptr;
            block.next = ptr::null_mut();
            // We don't need to set size here because we'll use the stored size from the block header.

            let mut current = self.head.load(Ordering::Acquire);
            let mut prev: *mut Block = ptr::null_mut();

            while !current.is_null() && current < block_ptr {
                prev = current;
                current = (*current).next;
            }

            self.insert_block(block_ptr, prev, current);
            self.coalesce();
        }
    }

    #[inline]
    fn insert_block(&mut self, block: *mut Block, prev: *mut Block, next: *mut Block) {
        unsafe {
            if prev.is_null() {
                (*block).next = self.head.load(Ordering::Acquire);
                self.head.store(block, Ordering::Release);
            } else {
                (*block).next = next;
                (*prev).next = block;
            }
        }
    }

    #[inline]
    fn coalesce(&mut self) {
        unsafe {
            let mut current = self.head.load(Ordering::Acquire);

            while !current.is_null() {
                let next = (*current).next;
                if next.is_null() {
                    break;
                }
                let current_end = current as usize + Block::header_size() + (*current).size;
                if current_end == next as usize {
                    (*current).coalesce_with_next(next);
                } else {
                    current = next;
                }
            }
        }
    }
}

static ARENA: SyncUnsafeCell<Arena> = SyncUnsafeCell::new(Arena::new());

pub struct FreeListAllocator;

impl FreeListAllocator {
    #[inline]
    fn arena(&self) -> *mut Arena {
        ARENA.get()
    }

    #[inline]
    fn ensure_initialized(&self) {
        unsafe {
            let arena = self.arena();
            if !(*arena).initialized {
                (*arena).init();
            }
        }
    }
}

unsafe impl GlobalAlloc for FreeListAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        if size == 0 {
            return align as *mut u8;
        }

        self.ensure_initialized();
        let arena = self.arena();
        (*arena).alloc(size, align)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if ptr.is_null() || layout.size() == 0 {
            return;
        }
        let arena = self.arena();
        (*arena).dealloc(ptr, layout.size());
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        unsafe {
            let new_layout = Layout::from_size_alignment_unchecked(new_size, layout.alignment());
            let new_ptr = self.alloc(new_layout);
            if !new_ptr.is_null() {
                let count = core::cmp::min(layout.size(), new_size);
                ptr::copy_nonoverlapping(ptr, new_ptr, count);
                self.dealloc(ptr, layout);
            }
            new_ptr
        }
    }
}
