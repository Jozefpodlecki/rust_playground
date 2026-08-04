
#![feature(allocator_api)]
#![allow(unused_unsafe)]

extern crate alloc;
use core::alloc::Layout;
use core::ptr;

use buddy_allocator::types::*;
use buddy_allocator::buddy::*;

macro_rules! create_allocator {
    ($size:expr, $min_block:expr) => {{
        const ARENA_SIZE: usize = $size;
        const MIN_BLOCK: usize = $min_block;
        const MAX_ORDER: usize = (ARENA_SIZE / MIN_BLOCK).ilog2() as usize;
        
        #[repr(align(4096))]
        struct AlignedArena([u8; ARENA_SIZE]);
        static mut ARENA: AlignedArena = AlignedArena([0; ARENA_SIZE]);

        #[allow(static_mut_refs)]
        unsafe { BuddyAllocatorImpl::<ARENA_SIZE, MIN_BLOCK, MAX_ORDER>(ARENA.0.as_mut_ptr()) }
    }};
}

#[test]
fn test_basic_alloc_dealloc() {

    let allocator = create_allocator!(1024, 32);
    let layout = Layout::from_size_align(64, 8).unwrap();
    let ptr = unsafe { allocator.alloc(layout) };
    assert!(!ptr.is_null());

    unsafe { ptr.write(0xAA) };
    assert_eq!(unsafe { ptr.read() }, 0xAA);
    unsafe { allocator.dealloc(ptr, layout) };
}

#[test]
fn test_split_and_merge() {
    let allocator = create_allocator!(1024, 32);
    let big_layout = Layout::from_size_align(200, 8).unwrap();
    let ptr_big = unsafe { allocator.alloc(big_layout) };
    assert!(!ptr_big.is_null());

    let small_layout = Layout::from_size_align(32, 8).unwrap();
    let ptr_small = unsafe { allocator.alloc(small_layout) };
    assert!(!ptr_small.is_null());

    unsafe { allocator.dealloc(ptr_small, small_layout) };

    let ptr_big2 = unsafe { allocator.alloc(big_layout) };
    assert!(!ptr_big2.is_null());

    unsafe { allocator.dealloc(ptr_big2, big_layout) };
    unsafe { allocator.dealloc(ptr_big, big_layout) };
}

#[test]
fn test_realloc_inplace() {

    let allocator = create_allocator!(4096, 32);
    let layout = Layout::from_size_align(500, 8).unwrap();
    let ptr = unsafe { allocator.alloc(layout) };
    assert!(!ptr.is_null());
    let block = BuddyBlock::<32>::from_ptr(ptr);
    let initial_order = unsafe { block.order() };

    let new_layout = Layout::from_size_align(600, 8).unwrap();
    let ptr2 = unsafe { allocator.realloc(ptr, layout, new_layout.size()) };
    assert_eq!(ptr, ptr2);

    let new_layout2 = Layout::from_size_align(1000, 8).unwrap();
    let ptr3 = unsafe { allocator.realloc(ptr2, new_layout, new_layout2.size()) };

    assert!(!ptr3.is_null());

    unsafe { allocator.dealloc(ptr3, new_layout2) };
}

#[test]
fn test_realloc_fallback() {
    let allocator = create_allocator!(1024, 32);
    let layout = Layout::from_size_align(64, 8).unwrap();
    let ptr = unsafe { allocator.alloc(layout) };
    assert!(!ptr.is_null());

    let new_layout = Layout::from_size_align(900, 8).unwrap();
    let ptr2 = unsafe { allocator.realloc(ptr, layout, new_layout.size()) };

    if !ptr2.is_null() {
        unsafe { allocator.dealloc(ptr2, new_layout) };
    } else {
        // If it fails, we still need to free the original.
        unsafe { allocator.dealloc(ptr, layout) };
    }
}

// #[test]
// fn test_many_small_allocations() {
//     let allocator = create_allocator!(1024, 32);
//     let small_layout = Layout::from_size_align(16, 8).unwrap(); // fits in 32-byte blocks
//     let mut ptrs = [ptr::null_mut(); 32];
//     for i in 0..32 {
//         let p = unsafe { allocator.alloc(small_layout) };
//         assert!(!p.is_null());
//         ptrs[i] = p;
//     }

//     for p in ptrs.iter() {
//         unsafe { allocator.dealloc(*p, small_layout) };
//     }

//     let big_layout = Layout::from_size_align(900, 8).unwrap();
//     let big_ptr = unsafe { allocator.alloc(big_layout) };
//     assert!(!big_ptr.is_null());
//     unsafe { allocator.dealloc(big_ptr, big_layout) };
// }

#[test]
fn test_alignment() {
    let allocator = create_allocator!(1024, 32);
    for align in [1, 2, 4, 8, 16].iter() {
        let size = 64;
        let layout = Layout::from_size_align(size, *align).unwrap();
        let ptr = unsafe { allocator.alloc(layout) };
        assert!(!ptr.is_null());
        assert_eq!(ptr as usize % align, 0);
        unsafe { allocator.dealloc(ptr, layout) };
    }
}

#[test]
fn test_zero_size_allocation() {
    let allocator = create_allocator!(1024, 32);
    let layout = Layout::from_size_align(0, 1).unwrap();
    let ptr = unsafe { allocator.alloc(layout) };

    if !ptr.is_null() {
        unsafe { allocator.dealloc(ptr, layout) };
    }

    let layout2 = Layout::from_size_align(32, 8).unwrap();
    let ptr2 = unsafe { allocator.alloc(layout2) };
    assert!(!ptr2.is_null());
    unsafe { allocator.dealloc(ptr2, layout2) };
}

#[test]
fn test_large_allocation_failure() {
    let allocator = create_allocator!(1024, 32);
    let layout = Layout::from_size_align(1200, 8).unwrap(); // larger than arena
    let ptr = unsafe { allocator.alloc(layout) };
    assert!(ptr.is_null());
}