use core::alloc::{Allocator, GlobalAlloc, Layout};
use toolkit::println;

use crate::allocator::*;

const MAX_ALLOCATIONS: usize = 1024;

#[derive(Clone, Copy)]
struct AllocEntry {
    ptr: *mut u8,
    layout: Layout,
}

impl AllocEntry {
    const fn null() -> Self {
        Self {
            ptr: core::ptr::null_mut(),
            layout: unsafe { Layout::from_size_align_unchecked(0, 1) },
        }
    }

    fn is_null(&self) -> bool {
        self.ptr.is_null()
    }
}

struct AllocStorage<const N: usize> {
    entries: [AllocEntry; N],
    len: usize,
}

impl<const N: usize> AllocStorage<N> {
    const fn new() -> Self {
        Self {
            entries: [AllocEntry::null(); N],
            len: 0,
        }
    }

    fn push(&mut self, entry: AllocEntry) -> Result<(), ()> {
        if self.len < N {
            self.entries[self.len] = entry;
            self.len += 1;
            Ok(())
        } else {
            Err(())
        }
    }

    fn remove(&mut self, index: usize) -> Option<AllocEntry> {
        if index >= self.len {
            return None;
        }
        let last_idx = self.len - 1;
        let entry = self.entries[index];
        self.entries[index] = self.entries[last_idx];
        self.entries[last_idx] = AllocEntry::null();
        self.len -= 1;
        Some(entry)
    }

    fn get(&self, index: usize) -> Option<&AllocEntry> {
        if index < self.len {
            Some(&self.entries[index])
        } else {
            None
        }
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut AllocEntry> {
        if index < self.len {
            Some(&mut self.entries[index])
        } else {
            None
        }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn capacity(&self) -> usize {
        N
    }

    fn iter(&self) -> core::slice::Iter<'_, AllocEntry> {
        self.entries[..self.len].iter()
    }

    fn clear(&mut self) {
        for i in 0..self.len {
            self.entries[i] = AllocEntry::null();
        }
        self.len = 0;
    }
}

pub fn run_stress_test<A: GlobalAlloc>(iterations: usize, allocator: A) {
    println!("=== Starting Buddy Allocator Stress Test ===");
    println!("Iterations: {}", iterations);

    let mut storage: AllocStorage<MAX_ALLOCATIONS> = AllocStorage::new();
    
    let mut rng: u64 = 0xDEADBEEF;
    let mut total_allocated = 0;
    let mut total_freed = 0;
    let mut alloc_count = 0;
    let mut free_count = 0;
    let mut realloc_count = 0;

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
                let ptr = unsafe { allocator.alloc(layout) };

                if !ptr.is_null() {
                    let entry = AllocEntry { ptr, layout };
                    if storage.push(entry).is_ok() {
                        total_allocated += size;
                        alloc_count += 1;
                        println!("  [{}] Allocated {} bytes at {:p} (active: {})",
                            iteration, size, ptr, storage.len());

                        unsafe {
                            for i in 0..size {
                                *ptr.add(i) = (i % 256) as u8;
                            }
                        }
                    } else {
                        println!("  [{}] Storage full, freeing allocated {} bytes at {:p}",
                            iteration, size, ptr);
                        unsafe { allocator.dealloc(ptr, layout) };
                    }
                } else {
                    println!("  [{}] Failed to allocate {} bytes", iteration, size);
                }
            }
            1 => {
                if !storage.is_empty() {
                    let idx = (rng % storage.len() as u64) as usize;
                    if let Some(entry) = storage.remove(idx) {
                        let size = entry.layout.size();
                        println!("  [{}] Freeing {} bytes at {:p}", iteration, size, entry.ptr);
                        unsafe { allocator.dealloc(entry.ptr, entry.layout) };
                        total_freed += size;
                        free_count += 1;
                    }
                }
            }
            _ => {
                if !storage.is_empty() {
                    let idx = (rng % storage.len() as u64) as usize;
                    if let Some(entry) = storage.get_mut(idx) {
                        let new_size = 1 + (rng % 8192) as usize;
                        println!("  [{}] Reallocating {:p} from {} to {} bytes",
                            iteration, entry.ptr, entry.layout.size(), new_size);

                        let new_ptr = unsafe { allocator.realloc(entry.ptr, entry.layout, new_size) };

                        if !new_ptr.is_null() {
                            println!("  [{}] Reallocated -> {:p}", iteration, new_ptr);
                            let old_size = entry.layout.size();
                            entry.ptr = new_ptr;
                            entry.layout = unsafe {
                                Layout::from_size_align_unchecked(new_size, entry.layout.align())
                            };
                            // Update total_allocated to reflect the size change
                            total_allocated = total_allocated - old_size + new_size;
                            realloc_count += 1;
                        } else {
                            println!("  [{}] Failed to reallocate to {} bytes", iteration, new_size);
                        }
                    }
                }
            }
        }

        if iteration % 50 == 0 && iteration > 0 {
            println!("--- Progress: {} iterations, allocs={}, frees={}, reallocs={}, active={} ---",
                iteration, alloc_count, free_count, realloc_count, storage.len());
        }
    }

    println!("Cleaning up remaining allocations... ({} remaining)", storage.len());
    for entry in storage.iter() {
        if !entry.is_null() {
            println!("  Freeing {:p} ({} bytes)", entry.ptr, entry.layout.size());
            unsafe { allocator.dealloc(entry.ptr, entry.layout) };
        }
    }
    storage.clear();

    println!("=== Stress Test Summary ===");
    println!("Total allocations: {}", alloc_count);
    println!("Total frees: {}", free_count);
    println!("Total reallocations: {}", realloc_count);
    println!("Total allocated: {} bytes", total_allocated);
    println!("Total freed: {} bytes", total_freed);
    println!("Remaining: {} bytes", total_allocated.saturating_sub(total_freed));
    println!("=== Stress Test Complete ===");
}