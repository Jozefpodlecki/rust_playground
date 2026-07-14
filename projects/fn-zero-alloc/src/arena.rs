use core::any::TypeId;
use core::cell::UnsafeCell;
use core::mem;
use core::ptr;

use toolkit::println;

use crate::handle::*;
use crate::types::*;

#[macro_export]
macro_rules! zero_alloc_arena {
    ($size:expr) => {
        use crate::arena::FunctionAllocatorInner;
        use crate::handle::*;

        static mut ARENA: FunctionAllocatorInner<{ $size }> = FunctionAllocatorInner::new();

        pub struct FunctionAllocator;

        impl FunctionAllocator {
            pub fn store<F, T>(closure: F) -> Option<&'static FnHandle>
            where
                F: FnOnce() -> T + Send + 'static,
            {
                unsafe {
                    let ptr = ARENA.store_closure(closure);
                    if ptr.is_null() {
                        None
                    } else {
                        Some(&*ptr)
                    }
                }
            }

            pub fn iter() -> HandlesIter {
                unsafe { ARENA.iter() }
            }

            pub fn count() -> usize {
                unsafe { ARENA.count() }
            }

            pub fn remove<const N: usize>(handle: &'static FnHandle) -> Option<StackedFunction<N>> {
                unsafe { ARENA.remove(handle.data_ptr.0) }
            }

            pub fn clear() {
                unsafe { ARENA.clear() }
            }

            pub fn debug_dump() {
                unsafe { ARENA.debug_dump(); }
            }
        }
    };
}

#[repr(C, align(16))]
pub struct FunctionAllocatorInner<const SIZE: usize> {
    data: UnsafeCell<[u8; SIZE]>,
    free_list: UnsafeCell<*mut ClosureHeader>,
}

impl<const SIZE: usize> FunctionAllocatorInner<SIZE> {
    pub const fn new() -> Self {
        Self {
            data: UnsafeCell::new([0; SIZE]),
            free_list: UnsafeCell::new(ptr::null_mut()),
        }
    }

    fn alloc(&self, size: usize, align: usize) -> *mut u8 {
        unsafe {
            let data_ptr = self.data.get() as *mut u8;
            let header_size = mem::size_of::<ClosureHeader>();
            let total_size = header_size + size;

            let mut offset = 0;
            let mut current = *self.free_list.get();
            let mut prev = ptr::null_mut();

            while !current.is_null() {
                let block_start = (current as usize) - (data_ptr as usize);

                // Try to place before this block
                let data_start_candidate = offset + header_size;
                let data_aligned = (data_start_candidate + align - 1) & !(align - 1);
                let header_start = data_aligned - header_size;
                let alloc_end = data_aligned + size;

                if alloc_end <= block_start {
                    // Fits before current block
                    break;
                }

                // Move past this block
                let block_end = block_start + header_size + (*current).meta.data_size;
                offset = block_end;
                prev = current;
                current = (*current).next;
            }

            // If we are at the end (current is null), check space from offset to SIZE
            if current.is_null() {
                let data_start_candidate = offset + header_size;
                let data_aligned = (data_start_candidate + align - 1) & !(align - 1);
                let alloc_end = data_aligned + size;
                if alloc_end > SIZE {
                    return ptr::null_mut();
                }
                // Place at end
                let header_offset = data_aligned - header_size;
                let header_ptr = data_ptr.add(header_offset) as *mut ClosureHeader;
                let data_aligned_ptr = data_ptr.add(data_aligned);

                ptr::write(
                    header_ptr,
                    ClosureHeader {
                        meta: ClosureMeta {
                            func_ptr: FuncPtr(ptr::null()),
                            data_ptr: DataPtr(ptr::null_mut()),
                            data_size: 0,
                            data_align: 0,
                            type_id: TypeId::of::<()>(),
                            drop_fn: DropFn(drop_nothing),
                        },
                        next: ptr::null_mut(),
                    },
                );

                if prev.is_null() {
                    *self.free_list.get() = header_ptr;
                } else {
                    (*prev).next = header_ptr;
                }

                return data_aligned_ptr;
            } else {
                // Place before current block
                let data_start_candidate = offset + header_size;
                let data_aligned = (data_start_candidate + align - 1) & !(align - 1);
                // Guaranteed: data_aligned + size <= block_start
                let header_offset = data_aligned - header_size;
                let header_ptr = data_ptr.add(header_offset) as *mut ClosureHeader;
                let data_aligned_ptr = data_ptr.add(data_aligned);

                ptr::write(
                    header_ptr,
                    ClosureHeader {
                        meta: ClosureMeta {
                            func_ptr: FuncPtr(ptr::null()),
                            data_ptr: DataPtr(ptr::null_mut()),
                            data_size: 0,
                            data_align: 0,
                            type_id: TypeId::of::<()>(),
                            drop_fn: DropFn(drop_nothing),
                        },
                        next: current,
                    },
                );

                if prev.is_null() {
                    *self.free_list.get() = header_ptr;
                } else {
                    (*prev).next = header_ptr;
                }

                return data_aligned_ptr;
            }
        }
    }

    fn dealloc(&self, ptr: *mut u8) {
        unsafe {
            let header_ptr = (ptr as usize - mem::size_of::<ClosureHeader>()) as *mut ClosureHeader;
            let size = (*header_ptr).meta.data_size;
            if size > 0 {
                let drop_fn = (*header_ptr).meta.drop_fn;
                let data_ptr = (*header_ptr).meta.data_ptr;
                (drop_fn.0)(data_ptr.0);
            }

            let mut current = *self.free_list.get();
            let mut prev = ptr::null_mut();

            while !current.is_null() && current < header_ptr {
                prev = current;
                current = (*current).next;
            }

            if prev.is_null() {
                (*header_ptr).next = *self.free_list.get();
                *self.free_list.get() = header_ptr;
            } else {
                (*header_ptr).next = (*prev).next;
                (*prev).next = header_ptr;
            }
        }
    }

    pub fn store_closure<F, T>(&self, closure: F) -> *mut FnHandle
    where
        F: FnOnce() -> T + Send + 'static,
    {
        unsafe {
            let closure_size = mem::size_of_val(&closure);
            let closure_align = mem::align_of_val(&closure);
            let handle_size = mem::size_of::<FnHandle>();
            let handle_align = mem::align_of::<FnHandle>();

            let align = closure_align.max(handle_align);
            let total_size = closure_size + handle_size;

            let data_ptr = self.alloc(total_size, align);
            if data_ptr.is_null() {
                return ptr::null_mut();
            }

            ptr::copy_nonoverlapping(&closure as *const _ as *const u8, data_ptr, closure_size);

            let handle_ptr = data_ptr.add(closure_size) as *mut FnHandle;
            let func_ptr = FuncPtr(call_closure::<F, T> as *const () as *const u8);

            let header_ptr = (data_ptr as usize - mem::size_of::<ClosureHeader>()) as *mut ClosureHeader;
            (*header_ptr).meta = ClosureMeta {
                func_ptr,
                data_ptr: DataPtr(data_ptr),
                data_size: total_size,
                data_align: align,
                type_id: TypeId::of::<F>(),
                drop_fn: DropFn(drop_closure::<F>),
            };

            let handle = FnHandle::new(func_ptr, DataPtr(data_ptr));
            ptr::write(handle_ptr, handle);

            handle_ptr
        }
    }

    pub fn remove<const N: usize>(&self, data_ptr: *mut u8) -> Option<StackedFunction<N>> {
        unsafe {
            let header_ptr = (data_ptr as usize - mem::size_of::<ClosureHeader>()) as *mut ClosureHeader;
            let meta = &(*header_ptr).meta;
            let size = meta.data_size;

            if size > N {
                return None;
            }

            let mut sf = StackedFunction::new();
            sf.func_ptr = meta.func_ptr.0;
            sf.drop_fn = meta.drop_fn.0;
            sf.size = size;
            ptr::copy_nonoverlapping(data_ptr, sf.data.as_mut_ptr(), size);
            self.dealloc(data_ptr);
            Some(sf)
        }
    }

    pub fn count(&self) -> usize {
        unsafe {
            let mut count = 0;
            let mut current = *self.free_list.get();
            while !current.is_null() {
                count += 1;
                current = (*current).next;
            }
            count
        }
    }

    pub fn iter(&self) -> HandlesIter {
        unsafe {
            HandlesIter {
                current: *self.free_list.get(),
            }
        }
    }

    pub unsafe fn clear(&self) {
        let mut current = *self.free_list.get();
        while !current.is_null() {
            let size = (*current).meta.data_size;
            if size > 0 {
                let drop_fn = (*current).meta.drop_fn;
                let data_ptr = (*current).meta.data_ptr;
                (drop_fn.0)(data_ptr.0);
            }
            current = (*current).next;
        }
        *self.free_list.get() = ptr::null_mut();
    }

    pub fn debug_dump(&self) {
        println!("=== Arena Debug Dump ===");
        println!("Total size: {} bytes", SIZE);
        let mut prev_end = 0;
        let mut idx = 0;
        for block in self.blocks() {
            let block_start = block.header_offset;
            let block_end = block.header_offset + mem::size_of::<ClosureHeader>() + block.data_size;
            // gap
            if block_start > prev_end {
                println!("  Gap: {} bytes (offset {} to {})", block_start - prev_end, prev_end, block_start);
            }
            println!(
                "  Block #{}: header at {}, data at {}, size {}, align {}",
                idx, block.header_offset, block.data_offset, block.data_size, block.data_align
            );
            prev_end = block_end;
            idx += 1;
        }
        if prev_end < SIZE {
            println!("  Free space at end: {} bytes (offset {} to {})", SIZE - prev_end, prev_end, SIZE);
        }
        println!("=== End Dump ===");
    }

    pub fn blocks(&self) -> BlocksIter {
        unsafe {
            BlocksIter::new(*self.free_list.get(), self.data.get() as *mut u8)
        }
    }
}

unsafe fn drop_nothing(_ptr: *mut u8) {}

unsafe fn call_closure<F, T>(data_ptr: *mut u8) -> T
where
    F: FnOnce() -> T,
{
    let closure: F = ptr::read(data_ptr as *const F);
    closure()
}

unsafe fn drop_closure<F>(data_ptr: *mut u8) {
    ptr::drop_in_place(data_ptr as *mut F);
}