use core::{
    alloc::Layout,
    ptr::{self, null_mut}
};

use winapi::um::winnt::PAGE_EXECUTE_READWRITE;

use crate::types::*;

const HEADER_SIZE: usize = size_of::<BlockHeader>();

#[allow(non_snake_case)]
#[unsafe(naked)]
unsafe extern "system" fn nt_protect_virtual_memory(
    handle: *mut core::ffi::c_void,
    base_addr: *mut *mut core::ffi::c_void,
    size: *mut usize,
    protect: u64,
    old: *mut u64,
) -> i32 {
    core::arch::naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x50",
        "syscall",
        "ret"
    );
}

pub fn init_arena() {
    unsafe {
        let arena = ArenaPtr::get();
        let mut old = 0;
        let mut addr_ptr = arena.as_ptr() as *mut _;
        let mut size = arena.size();

        nt_protect_virtual_memory(
            -1isize as *mut _,
            &mut addr_ptr, 
            &mut size,
            PAGE_EXECUTE_READWRITE as _,
            &mut old,
        );

        let head = arena.as_atomic_block_header();
        let block = arena.add(HEADER_SIZE).as_block_header();
        
        block.write(BlockHeader {
            size: size - HEADER_SIZE,
            next: AtomicBlockHeader::default(),
        });
        
        head.write(AtomicBlockHeader::new(block));
    }
}

unsafe fn update_block_link(
    head: *mut AtomicBlockHeader,
    previous: BlockPtr,
    current: BlockPtr,
    new: BlockPtr,
) -> bool {
    if previous.is_null() {
        (*head).cas(current, new)
    } else {
        (*previous.as_ptr()).next.cas(current, new)
    }
}

unsafe fn split_block(
    current: BlockPtr,
    next: BlockPtr,
    total_needed: usize,
    block_start: usize,
) -> BlockPtr {
    let remainder = current.size() - total_needed;
    let remainder_start = block_start + total_needed;
    let rem_block = BlockPtr(remainder_start as *mut BlockHeader);
    
    rem_block.write(BlockHeader {
        size: remainder - HEADER_SIZE,
        next: AtomicBlockHeader::new(next),
    });
    
    rem_block
}

unsafe fn try_alloc_from_block(
    head: *mut AtomicBlockHeader,
    current: BlockPtr,
    previous: BlockPtr,
    size: usize,
    align: usize,
) -> Option<(AllocPtr, bool)> {
    let block_size = current.size();
    let block_start = current.as_usize();
    let aligned_start = (block_start + HEADER_SIZE + align - 1) & !(align - 1);
    let aligned_start_ptr = aligned_start as *mut u8;
    let aligned_end = aligned_start + size;
    let total_needed = aligned_end - block_start;

    if total_needed > block_size {
        return None;
    }

    let next = current.next().load();

    if block_size - total_needed > HEADER_SIZE {
        let rem_block = split_block(current, next, total_needed, block_start);
        
        if update_block_link(head, previous, current, rem_block) {
            return Some((AllocPtr(aligned_start_ptr), true));
        }
    } else {
        if update_block_link(head, previous, current, next) {
            return Some((AllocPtr(aligned_start_ptr), true));
        }
    }

    Some((AllocPtr(null_mut()), false))
}

unsafe fn alloc_from_arena(arena: ArenaPtr, layout: Layout) -> *mut u8 {
    let head = arena.as_atomic_block_header();
    let size = layout.size();
    let align = layout.align();

    loop {
        let mut current = (*head).load();
        let mut previous = BlockPtr::null();

        while !current.is_null() {
            let next = (*current.as_ptr()).next.load();

            match try_alloc_from_block(head, current, previous, size, align) {
                Some((ptr, true)) => return ptr.as_ptr(),
                Some((_, false)) => break,
                None => {
                    previous = current;
                    current = next;
                }
            }
        }
    }
}

unsafe fn dealloc_to_arena(arena: ArenaPtr, ptr: *mut u8, size: usize) {
    let head = arena.as_atomic_block_header();
    let block_start = BlockPtr((ptr as usize - HEADER_SIZE) as *mut BlockHeader);

    let new_block = BlockHeader {
        size: size + HEADER_SIZE,
        next: AtomicBlockHeader::default(),
    };

    loop {
        let current_head = (*head).load();
        new_block.next.store(current_head);

        if (*head).cas(current_head, block_start) {
            block_start.write(new_block);
            return;
        }
    }
}

#[rustc_std_internal_symbol]
#[rustc_allocator]
pub unsafe fn __rust_alloc(size: usize, align: core::mem::Alignment) -> *mut u8 {
    let arena = ArenaPtr::get();
    let layout = Layout::from_size_align_unchecked(size, align.as_usize());
    alloc_from_arena(arena, layout)
}

#[rustc_std_internal_symbol]
#[rustc_deallocator]
unsafe fn __rust_dealloc(ptr: *mut u8, size: usize, _align: core::mem::Alignment) {
    let arena = ArenaPtr::get();
    dealloc_to_arena(arena, ptr, size);
}

#[rustc_std_internal_symbol]
#[rustc_reallocator]
pub unsafe fn __rust_realloc(ptr: *mut u8, size: usize, align: core::mem::Alignment, new_size: usize) -> *mut u8 {
    if new_size <= size {
        return ptr;
    }

    let new_ptr = __rust_alloc(new_size, align);
    if !new_ptr.is_null() {
        ptr::copy_nonoverlapping(ptr, new_ptr, size);
        let arena = ArenaPtr::get();
        dealloc_to_arena(arena, ptr, size);
    }
    new_ptr
}

#[rustc_std_internal_symbol]
#[rustc_allocator_zeroed]
pub unsafe fn __rust_alloc_zeroed(size: usize, align: core::mem::Alignment) -> *mut u8 {
    let ptr = __rust_alloc(size, align);
    if !ptr.is_null() {
        ptr::write_bytes(ptr, 0, size);
    }
    ptr
}