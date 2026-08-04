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

        struct TestAllocator(BuddyAllocatorImpl<ARENA_SIZE, MIN_BLOCK, MAX_ORDER>);

        impl TestAllocator {
            pub fn alloc_with_size_and_align(&self, size: usize, align: usize) -> *mut u8 {
                let layout = Layout::from_size_align(size, align).unwrap();
                self.0.alloc(layout)
            }

            pub fn dealloc_with_size_and_align(&self, ptr: *mut u8, size: usize, align: usize) {
                let layout = Layout::from_size_align(size, align).unwrap();
                self.0.dealloc(ptr, layout)
            }

            pub fn alloc_layout(&self, layout: Layout) -> *mut u8 {
                self.0.alloc(layout)
            }

            pub fn dealloc_layout(&self, ptr: *mut u8, layout: Layout) {
                self.0.dealloc(ptr, layout)
            }

            pub fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
                self.0.realloc(ptr, layout, new_size)
            }
        }

        #[allow(static_mut_refs)]
        TestAllocator(unsafe { BuddyAllocatorImpl::<ARENA_SIZE, MIN_BLOCK, MAX_ORDER>(ARENA.0.as_mut_ptr()) })
    }};
}


#[test]
fn test_basic_alloc_dealloc() {
    let allocator = create_allocator!(1024, 32);
    let ptr = allocator.alloc_with_size_and_align(64, 8);
    assert!(!ptr.is_null());
    unsafe { ptr.write(0xAA) };
    assert_eq!(unsafe { ptr.read() }, 0xAA);
    allocator.dealloc_with_size_and_align(ptr, 64, 8);
}

#[test]
fn test_split_and_merge() {
    let allocator = create_allocator!(1024, 32);
    let ptr_big = allocator.alloc_with_size_and_align(200, 8);
    assert!(!ptr_big.is_null());
    let ptr_small = allocator.alloc_with_size_and_align(32, 8);
    assert!(!ptr_small.is_null());
    allocator.dealloc_with_size_and_align(ptr_small, 32, 8);
    let ptr_big2 = allocator.alloc_with_size_and_align(200, 8);
    assert!(!ptr_big2.is_null());
    allocator.dealloc_with_size_and_align(ptr_big2, 200, 8);
    allocator.dealloc_with_size_and_align(ptr_big, 200, 8);
}

#[test]
fn test_realloc_inplace() {
    let allocator = create_allocator!(4096, 32);
    let layout_500 = Layout::from_size_align(500, 8).unwrap();
    let ptr_500 = allocator.alloc_layout(layout_500);
    assert!(!ptr_500.is_null());
    let layout_600 = Layout::from_size_align(600, 8).unwrap();
    let ptr_600 = allocator.realloc(ptr_500, layout_500, layout_600.size());
    assert_eq!(ptr_500, ptr_600);
    let layout_1000 = Layout::from_size_align(1000, 8).unwrap();
    let ptr_1000 = allocator.realloc(ptr_600, layout_600, layout_1000.size());
    assert!(!ptr_1000.is_null());
    allocator.dealloc_layout(ptr_1000, layout_1000);
}

#[test]
fn test_realloc_fallback() {
    let allocator = create_allocator!(1024, 32);
    let layout_64 = Layout::from_size_align(64, 8).unwrap();
    let ptr_64 = allocator.alloc_layout(layout_64);
    assert!(!ptr_64.is_null());
    let layout_900 = Layout::from_size_align(900, 8).unwrap();
    let ptr_900 = allocator.realloc(ptr_64, layout_64, layout_900.size());
    if !ptr_900.is_null() {
        allocator.dealloc_layout(ptr_900, layout_900);
    } else {
        allocator.dealloc_layout(ptr_64, layout_64);
    }
}

#[test]
fn test_realloc_shrink() {
    let allocator = create_allocator!(4096, 32);
    let layout_big = Layout::from_size_align(1000, 8).unwrap();
    let ptr_big = allocator.alloc_layout(layout_big);
    assert!(!ptr_big.is_null());
    let layout_small = Layout::from_size_align(100, 8).unwrap();
    let ptr_small = allocator.realloc(ptr_big, layout_big, layout_small.size());
    assert_eq!(ptr_big, ptr_small);
    allocator.dealloc_layout(ptr_small, layout_small);
}

#[test]
fn test_realloc_same_size() {
    let allocator = create_allocator!(1024, 32);
    let layout = Layout::from_size_align(64, 8).unwrap();
    let ptr = allocator.alloc_layout(layout);
    assert!(!ptr.is_null());
    let ptr_realloc = allocator.realloc(ptr, layout, layout.size());
    assert_eq!(ptr, ptr_realloc);
    allocator.dealloc_layout(ptr_realloc, layout);
}

#[test]
fn test_max_order_allocation() {
    let allocator = create_allocator!(4096, 32);
    // 4096-byte arena with MIN_BLOCK=32, the max block size is 2048 (order 6)
    // The header size for a block is 16 bytes, so the maximum user-usable size is 2048 - 16 = 2032 bytes
    let max_size = 2032;
    let ptr = allocator.alloc_with_size_and_align(max_size, 8);
    assert!(!ptr.is_null());
    allocator.dealloc_with_size_and_align(ptr, max_size, 8);
}

#[test]
fn test_allocation_above_power_of_two() {
    let allocator = create_allocator!(4096, 32);
    for size in [257, 513, 1025, 2000].iter() {
        let ptr = allocator.alloc_with_size_and_align(*size, 8);
        assert!(!ptr.is_null());
        allocator.dealloc_with_size_and_align(ptr, *size, 8);
    }
}

#[test]
fn test_many_small_allocations_and_full_coalesce() {
    let allocator = create_allocator!(4096, 32);
    let mut ptrs = [ptr::null_mut(); 64];
    let mut count = 0;
    for i in 0..64 {
        let p = allocator.alloc_with_size_and_align(16, 8);
        if p.is_null() { break; }
        ptrs[i] = p;
        count += 1;
    }
    assert!(count > 0);
    for i in 0..count {
        allocator.dealloc_with_size_and_align(ptrs[i], 16, 8);
    }
    let big_ptr = allocator.alloc_with_size_and_align(1500, 8);
    assert!(!big_ptr.is_null());
    allocator.dealloc_with_size_and_align(big_ptr, 1500, 8);
}

#[test]
fn test_alignment() {
    let allocator = create_allocator!(1024, 32);
    for align in [1, 2, 4, 8, 16].iter() {
        let ptr = allocator.alloc_with_size_and_align(64, *align);
        assert!(!ptr.is_null());
        assert_eq!(ptr as usize % align, 0);
        allocator.dealloc_with_size_and_align(ptr, 64, *align);
    }
}

#[test]
fn test_zero_size_allocation() {
    let allocator = create_allocator!(1024, 32);
    let ptr = allocator.alloc_with_size_and_align(0, 1);
    if !ptr.is_null() {
        allocator.dealloc_with_size_and_align(ptr, 0, 1);
    }
    let ptr2 = allocator.alloc_with_size_and_align(32, 8);
    assert!(!ptr2.is_null());
    allocator.dealloc_with_size_and_align(ptr2, 32, 8);
}

#[test]
fn test_large_allocation_failure() {
    let allocator = create_allocator!(1024, 32);
    let ptr = allocator.alloc_with_size_and_align(1200, 8);
    assert!(ptr.is_null());
}