
#[macro_export]
macro_rules! buddy_allocator {
    ($size:expr) => {
        use core::alloc::{GlobalAlloc, Layout};
        use toolkit::Mutex;
        use $crate::buddy::BuddyAllocator as BuddyAllocatorImpl;

        pub const HEAP_SIZE: usize = $size;

        static mut HEAP_MEMORY: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
        static ALLOCATOR: Mutex<BuddyAllocatorImpl<HEAP_SIZE>> = unsafe { 
            Mutex::new(BuddyAllocatorImpl(HEAP_MEMORY.as_mut_ptr())) 
        };

        #[global_allocator]
        static GLOBAL_ALLOCATOR: BuddyAllocator = BuddyAllocator;

        pub struct BuddyAllocator;

        unsafe impl GlobalAlloc for BuddyAllocator {
            unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
                ALLOCATOR.lock().alloc(layout)
            }

            unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
                ALLOCATOR.lock().dealloc(ptr, layout)
            }

            unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
                unsafe { ALLOCATOR.lock().realloc(ptr, layout, new_size) }
            }
        }

        impl BuddyAllocator {
            pub fn dump_free_lists(&self) {
                unsafe {
                    let allocator = ALLOCATOR.lock();
                    toolkit::println!("=== Buddy Allocator Free Lists ===");
                    toolkit::println!("Total Arena Size: {} bytes", HEAP_SIZE);
                    
                    let mut total_free = 0;
                    let mut total_blocks = 0;
                    
                    for order in 0..=$crate::buddy::MAX_ORDER {
                        let head = allocator.free_list(order);
                        let mut block = head.next();
                        let mut count = 0;
                        let mut size_sum = 0;
                        
                        while !block.is_null() {
                            count += 1;
                            let size = block.size();
                            size_sum += size;
                            block = block.next();
                        }
                        
                        if count > 0 {
                            let block_size = $crate::buddy::MIN_BLOCK_SIZE << order;
                            toolkit::println!("  Order {} ({} bytes): {} blocks, {} total bytes", 
                                order, block_size, count, size_sum);
                            total_free += size_sum;
                            total_blocks += count;
                        }
                    }
                    
                    toolkit::println!("Total Free: {} bytes in {} blocks", total_free, total_blocks);
                    toolkit::println!("=====================================");
                }
            }

            pub fn dump_all_blocks(&self) {
                unsafe {
                    let allocator = ALLOCATOR.lock();
                    toolkit::println!("=== All Blocks ===");
                    
                    let mut block = allocator.first_block();
                    let end = allocator.0.add(HEAP_SIZE);
                    let mut index = 0;
                    
                    while (block as *mut u8) < end {
                        let info = $crate::buddy::BuddyBlockInfo {
                            ptr: block,
                            order: block.order(),
                            size: block.size(),
                            free: block.free(),
                        };
                        
                        toolkit::println!("  Block {}: {:p} order={} size={} {}", 
                            index, 
                            info.ptr(), 
                            info.order(), 
                            info.size(), 
                            if info.free() { "FREE" } else { "USED" }
                        );
                        
                        let next_block = block.add_offset(block.size());
                        if next_block == block {
                            break;
                        }
                        block = next_block;
                        index += 1;
                    }
                    
                    toolkit::println!("Total Blocks: {}", index + 1);
                    toolkit::println!("==================");
                }
            }

            pub fn validate(&self) -> bool {
                unsafe {
                    let allocator = ALLOCATOR.lock();
                    let mut valid = true;
                    
                    let mut block = allocator.first_block();
                    let end = allocator.0.add(HEAP_SIZE);
                    let mut total_size = 0;
                    
                    while (block as *mut u8) < end {
                        let size = block.size();
                        total_size += size;
                        
                        if size == 0 || !size.is_power_of_two() {
                            toolkit::println!("ERROR: Invalid block size {} at {:?}", size, block);
                            valid = false;
                        }
                        
                        if block.free() {
                            let order = block.order() as usize;
                            let expected_size = $crate::buddy::MIN_BLOCK_SIZE << order;
                            if size != expected_size {
                                toolkit::println!("ERROR: Size {} doesn't match order {} at {:?}", size, order, block);
                                valid = false;
                            }
                        }
                        
                        let next = block.add_offset(size);
                        if next == block {
                            break;
                        }
                        block = next;
                    }
                    
                    if total_size != HEAP_SIZE - $crate::buddy::BuddyBlock::size_of() {
                        toolkit::println!("ERROR: Total block size {} doesn't match arena size {}", total_size, HEAP_SIZE);
                        valid = false;
                    }
                    
                    valid
                }
            }

            pub fn fragmentation_report(&self) {
                unsafe {
                    let allocator = ALLOCATOR.lock();
                    let mut free_blocks = 0;
                    let mut used_blocks = 0;
                    let mut total_free = 0;
                    let mut total_used = 0;
                    let mut max_free = 0;
                    
                    let mut block = allocator.first_block();
                    let end = allocator.0.add(HEAP_SIZE);
                    
                    while (block as *mut u8) < end {
                        let size = block.size();
                        if block.free() {
                            free_blocks += 1;
                            total_free += size;
                            if size > max_free {
                                max_free = size;
                            }
                        } else {
                            used_blocks += 1;
                            total_used += size;
                        }
                        
                        let next = block.add_offset(size);
                        if next == block {
                            break;
                        }
                        block = next;
                    }
                    
                    let total = total_free + total_used;
                    let fragmentation = if total > 0 { 
                        (total_free - max_free) * 100 / total 
                    } else { 0 };
                    
                    toolkit::println!("=== Fragmentation Report ===");
                    toolkit::println!("Total Memory: {} bytes", total);
                    toolkit::println!("Used: {} bytes ({} blocks)", total_used, used_blocks);
                    toolkit::println!("Free: {} bytes ({} blocks)", total_free, free_blocks);
                    toolkit::println!("Largest Free Block: {} bytes", max_free);
                    toolkit::println!("External Fragmentation: {}%", fragmentation);
                    if free_blocks > 0 {
                        let avg_free = total_free / free_blocks;
                        toolkit::println!("Average Free Block: {} bytes", avg_free);
                    }
                    toolkit::println!("=============================");
                }
            }
        }

        
        pub fn buddy_dump() {
            GLOBAL_ALLOCATOR.dump_free_lists();
        }

        pub fn buddy_fragmentation() {
            GLOBAL_ALLOCATOR.fragmentation_report();
        }

        pub fn buddy_validate() {
            if GLOBAL_ALLOCATOR.validate() {
                toolkit::println!("Buddy allocator validation: OK");
            } else {
                toolkit::println!("Buddy allocator validation: FAILED");
            }
        }
    };
}