#![no_std]
#![no_main]
#![windows_subsystem = "console"]

use core::{panic::PanicInfo, ptr};

use ntapi::ntrtl::{RtlAllocateHeap, RtlCreateHeap, RtlDestroyHeap};
use toolkit::{ProcessEnvironmentBlock, ProcessMemoryReader, println};

use crate::{flags::HeapFlags, heap::{HeapParameters, NtHeap}};

extern crate builtins;

mod flags;
mod error;
mod heap;


#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}


#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {
    let default_heap = NtHeap::from_peb();

    println!("Default heap {:p}", default_heap.handle());
    
    // Example 1: Create custom heap
    let heap = NtHeap::new(HeapParameters::default()).unwrap();
    println!("Custom heap {:p}", heap.handle());
    
    // Example 2: Allocate with zeroed memory
    let ptr = heap.allocate_zeroed(1024).unwrap();
    println!("Allocated {} bytes at {:p}", 1024, ptr);
    
    // Example 3: Allocate with flags
    let ptr2 = heap.allocate(512, HeapFlags::NO_SERIALIZE | HeapFlags::ALIGN_16).unwrap();
    println!("Allocated {} bytes with flags at {:p}", 512, ptr2);
    
    // Example 4: Reallocate
    let ptr3 = heap.reallocate(ptr, 2048, HeapFlags::ZERO_MEMORY).unwrap();
    println!("Reallocated to {} bytes at {:p}", 2048, ptr3);
    
    // Example 5: Get heap size
    let size = heap.size(ptr3, HeapFlags::NONE);
    println!("Heap size: {}", size);
    
    // Example 6: Multiple allocations
    let ptrs = heap.multiple_alloc::<100>(64, HeapFlags::ZERO_MEMORY).unwrap();
    println!("Allocated 10 blocks of 64 bytes");
    
    // Example 7: User value
    let ptr4 = heap.allocate(64, HeapFlags::NONE).unwrap();
    heap.set_user_value(ptr4, 0xdeadbeef as *mut _);

    match heap.get_user_value(ptr4) {
        Some(user_value) => println!("User value: {:p}", user_value),
        None => println!("Failed to get user value"),
    }
    
    // Example 8: Get process heaps
    {
        let mut count = 0;
        let mut heaps = NtHeap::process_heaps::<4>();
        let iter_heap = heaps.next().unwrap();
        println!("Heap {}: {:p} is_lfh={}", count, iter_heap.handle(), iter_heap.is_lfh());
        let iter_heap = heaps.next().unwrap();
        println!("Heap {}: {:p} is_lfh={}", count, iter_heap.handle(), iter_heap.is_lfh());
        
        // TO-DO STATUS_ACCESS_VIOLATION
        let iter_heap = heaps.next().unwrap();
        println!("Heap {}: {:p} is_lfh={}", count, iter_heap.handle(), iter_heap.is_lfh());

        // for iter_heap in heaps {
        //     count += 1;
        //     println!("Heap {}: {:p}", count, iter_heap.handle());
        // }
        // println!("Process has {} heaps", count);
    }
    
    println!("Compacting heap");
    // Example 9: Compact heap
    let compacted = heap.compact(HeapFlags::NONE);
    println!("Compacted {} bytes", compacted);
    
    // Example 10: Validate heap
    let valid = heap.validate(HeapFlags::NONE, ptr::null_mut());
    println!("Heap valid: {}", valid);
    
    // Example 11: Lock/Unlock
    heap.lock();
    // ... critical section ...
    heap.unlock();
    println!("Heap locked/unlocked");
    
    // Free everything
    heap.free(ptr3, HeapFlags::NONE);
    heap.free(ptr2, HeapFlags::NONE);
    heap.free(ptr4, HeapFlags::NONE);
    heap.multiple_free(&mut ptrs.clone(), HeapFlags::NONE);
    
    // Heap destroyed on drop
    println!("All tests passed!");
    0
}