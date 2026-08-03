use alloc::vec::Vec;
use core::alloc::{GlobalAlloc, Layout};
use toolkit::println;

use crate::{BuddyAllocator, GLOBAL_ALLOCATOR};

pub fn run_stress_test(iterations: usize) {
    println!("=== Starting Buddy Allocator Stress Test ===");
    println!("Iterations: {}", iterations);
    
    let mut allocations: Vec<(*mut u8, Layout)> = Vec::new();
    let mut rng: u64 = 0xDEADBEEF;
    let mut total_allocated = 0;
    let mut total_freed = 0;
    
    for iteration in 0..iterations {
        let operation = (rng % 3) as u8;
        rng = rng.wrapping_mul(0x41C64E6D).wrapping_add(0x3039);
        
        match operation {
            0 => {
                let size = 1 + (rng % 4096) as usize;
                rng = rng.wrapping_mul(0x41C64E6D).wrapping_add(0x3039);
                let align = match rng % 4 {
                    0 => 1,
                    1 => 2,
                    2 => 4,
                    _ => 8,
                };
                
                let layout = unsafe { Layout::from_size_align_unchecked(size, align) };
                let ptr = unsafe { GLOBAL_ALLOCATOR.alloc(layout) };
                
                if !ptr.is_null() {
                    allocations.push((ptr, layout));
                    total_allocated += size;
                    println!("  [{}] Allocated {} bytes at {:p}", iteration, size, ptr);
                    
                    unsafe {
                        for i in 0..size {
                            *ptr.add(i) = (i % 256) as u8;
                        }
                    }
                } else {
                    println!("  [{}] Failed to allocate {} bytes", iteration, size);
                }
            }
            1 => {
                if !allocations.is_empty() {
                    let idx = (rng % allocations.len() as u64) as usize;
                    let (ptr, layout) = allocations.remove(idx);
                    let size = layout.size();
                    unsafe {
                        GLOBAL_ALLOCATOR.dealloc(ptr, layout);
                    }
                    total_freed += size;
                    println!("  [{}] Freed {} bytes at {:p}", iteration, size, ptr);
                }
            }
            _ => {
                if !allocations.is_empty() {
                    let idx = (rng % allocations.len() as u64) as usize;
                    let (ptr, layout) = &mut allocations[idx];
                    let new_size = 1 + (rng % 8192) as usize;
                    
                    let new_ptr = unsafe { 
                        GLOBAL_ALLOCATOR.realloc(*ptr, *layout, new_size) 
                    };
                    
                    if !new_ptr.is_null() {
                        println!("  [{}] Reallocated {} -> {} bytes at {:p} -> {:p}", 
                            iteration, layout.size(), new_size, *ptr, new_ptr);
                        *ptr = new_ptr;
                        *layout = unsafe { Layout::from_size_align_unchecked(new_size, layout.align()) };
                    } else {
                        println!("  [{}] Failed to reallocate to {} bytes", iteration, new_size);
                    }
                }
            }
        }
        
        if iteration % 50 == 0 && iteration > 0 {
            println!("--- Progress: {} iterations ---", iteration);
            // BuddyAllocator::buddy_dump!();
            // BuddyAllocator::buddy_fragmentation!();
        }
    }
    
    println!("Cleaning up remaining allocations...");
    for (ptr, layout) in allocations {
        unsafe {
            GLOBAL_ALLOCATOR.dealloc(ptr, layout);
        }
    }
    
    println!("=== Stress Test Summary ===");
    println!("Total allocated: {} bytes", total_allocated);
    println!("Total freed: {} bytes", total_freed);
    println!("Remaining: {} bytes", total_allocated - total_freed);
    println!("=== Stress Test Complete ===");
    
    // BuddyAllocator::buddy_dump!();
    // BuddyAllocator::buddy_fragmentation!();
    // BuddyAllocator::buddy_validate!();
}