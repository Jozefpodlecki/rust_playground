use core::any::TypeId;
use core::cell::UnsafeCell;
use core::mem;
use core::ptr;

use toolkit::Mutex;
use toolkit::println;

use crate::packet::Packet;

// static mut ARENA: FunctionAllocatorInner<1024> = FunctionAllocatorInner::new();

// pub struct FunctionAllocator;

// impl FunctionAllocator {
//     pub fn store_closure<F, T>(closure: F) -> Option<ArenaPtr>
//     where
//         F: FnOnce() -> T + Send + 'static,
//     {
//         unsafe {
//             let ptr = ARENA.store_closure(Closure(closure));
//             if ptr.is_null() {
//                 None
//             } else {
//                 Some(ptr)
//             }
//         }
//     }

//     pub fn alloc_packet<T>(closure_ptr: ArenaPtr) -> *mut Packet<T> {
//         unsafe {
//             ARENA.alloc_packet(closure_ptr)
//         }
//     }

//     pub fn remove_packet<T>(packet_ptr: *mut Packet<T>) {
//         unsafe {
//             ARENA.remove_packet(packet_ptr);
//         }
//     }

//     pub fn remove_raw(ptr: ArenaPtr) {
//         unsafe {
//             ARENA.remove_raw(ptr);
//         }
//     }

//     pub fn dump() {
//         unsafe {
//             ARENA.debug_dump();
//         }
//     }

//     pub fn clear() {
//         unsafe {
//             ARENA.clear();
//         }
//     }
// }

#[macro_export]
macro_rules! thread_allocator {
    ($size:expr) => {
        use core::{fmt, ptr};
        use toolkit::{syscalls::{NtClose, NtWaitForSingleObject}};
        use winapi::{shared::{ntdef::NTSTATUS, ntstatus::{STATUS_SUCCESS, STATUS_TIMEOUT}}, um::winnt::{HANDLE, THREAD_ALL_ACCESS, LARGE_INTEGER}};
        use toolkit::syscalls::NtCreateThreadEx;
        use ntapi::ntpsapi::NtCurrentProcess;
        use crate::arena::*;
        use crate::packet::Packet;

        static mut ARENA: FunctionAllocatorInner<{ $size }> = FunctionAllocatorInner::new();

        pub struct FunctionAllocator;

        impl FunctionAllocator {
            pub fn store_closure<F, T>(closure: F) -> Option<ArenaPtr>
            where
                F: FnOnce() -> T + Send + 'static,
            {
                unsafe {
                    let ptr = ARENA.store_closure(Closure::new(closure));
                    if ptr.is_null() {
                        None
                    } else {
                        Some(ptr)
                    }
                }
            }

            pub fn alloc_packet<T>(closure_ptr: ArenaPtr) -> *mut Packet<T> {
                unsafe { ARENA.alloc_packet(closure_ptr) }
            }

            pub fn remove_packet<T>(packet_ptr: *mut Packet<T>) {
                unsafe { ARENA.remove_packet(packet_ptr) }
            }

            pub fn remove_raw(ptr: ArenaPtr) {
                unsafe { ARENA.remove_raw(ptr) }
            }

            pub fn dump() {
                unsafe { ARENA.debug_dump() }
            }

            pub fn clear() {
                unsafe { ARENA.clear() }
            }
        }

        #[derive(Debug)]
        pub enum ThreadError {
            Timeout,
            CreationFailed(NTSTATUS),
            WaitFailed(NTSTATUS),
            AllocationFailed,
            NoResult,
        }

        impl fmt::Display for ThreadError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    ThreadError::Timeout => write!(f, "Thread timed out"),
                    ThreadError::CreationFailed(status) => write!(f, "Thread creation failed: {}", status),
                    ThreadError::WaitFailed(status) => write!(f, "Thread wait failed: {}", status),
                    ThreadError::AllocationFailed => write!(f, "Allocation failed"),
                    ThreadError::NoResult => write!(f, "No result available"),
                }
            }
        }

        pub struct JoinHandle<T> {
            pub handle: HANDLE,
            pub packet: *mut Packet<T>,
        }

        impl<T> JoinHandle<T> {
            pub fn is_finished(&self) -> bool {
                unsafe {
                    if self.packet.is_null() {
                        return true;
                    }
                    (*self.packet).is_finished()
                }
            }

            pub fn join_timeout(&self, timeout_ms: u32) -> Result<T, ThreadError> {
                unsafe {
                    let mut timeout: LARGE_INTEGER = unsafe { core::mem::zeroed() };
                    *timeout.QuadPart_mut() = -(timeout_ms as i64 * 10_000);

                    let status = NtWaitForSingleObject(
                        self.handle,
                        0,
                        &mut timeout,
                    );

                    if status == STATUS_TIMEOUT {
                        return Err(ThreadError::Timeout);
                    }

                    if status != STATUS_SUCCESS {
                        return Err(ThreadError::WaitFailed(status));
                    }

                    if self.packet.is_null() {
                        return Err(ThreadError::NoResult);
                    }

                    let result = (*self.packet).take_result()
                        .ok_or(ThreadError::NoResult)?;

                    Ok(result)
                }
            }

            pub fn join(mut self) -> Result<T, ThreadError> {
                unsafe {
                    let status = NtWaitForSingleObject(
                        self.handle,
                        0,
                        ptr::null_mut(),
                    );

                    if status != STATUS_SUCCESS {
                        return Err(ThreadError::WaitFailed(status));
                    }
                    
                    NtClose(self.handle);

                    if self.packet.is_null() {
                        return Err(ThreadError::NoResult);
                    }

                    let result = (*self.packet).take_result()
                        .ok_or(ThreadError::NoResult)?;

                    FunctionAllocator::remove_packet(self.packet);

                    Ok(result)
                }
            }
        }

        unsafe extern "system" fn thread_entry<F, T: 'static>(param_ptr: *mut winapi::ctypes::c_void) -> u32
        where
            F: FnOnce() -> T + 'static,
        {
            unsafe {
                let packet = Packet::<T>::from_ptr(param_ptr);
                let closure: F = packet.take_closure();
                let result = closure();
                packet.store_result(result);

                0
            }
        }

        pub fn create_thread<F, T>(func: F) -> Result<JoinHandle<T>, ThreadError>
        where
            F: FnOnce() -> T + Send + 'static,
            T: Send + 'static,
        {
            let closure_ptr = FunctionAllocator::store_closure(func)
                .ok_or(ThreadError::AllocationFailed)?;
            
            let packet = FunctionAllocator::alloc_packet::<T>(closure_ptr);

            if packet.is_null() {
                FunctionAllocator::remove_raw(closure_ptr);
                return Err(ThreadError::AllocationFailed);
            }
            
            let mut handle: HANDLE = ptr::null_mut();
            
            let status = unsafe {
                NtCreateThreadEx(
                    &mut handle,
                    THREAD_ALL_ACCESS,
                    ptr::null_mut(),
                    NtCurrentProcess,
                    thread_entry::<F, T> as *const () as _,
                    packet as *mut _,
                    0,
                    0,
                    0,
                    0,
                    ptr::null_mut(),
                )
            };
            
            if status != STATUS_SUCCESS {
                FunctionAllocator::remove_packet(packet);
                return Err(ThreadError::CreationFailed(status));
            }
            
            Ok(JoinHandle {
                handle,
                packet,
            })
        }

        impl<T> Drop for Packet<T> {
            fn drop(&mut self) {
                let mut guard = self.0.lock();
                if !guard.closure_ptr.is_null() {
                    unsafe {
                        FunctionAllocator::remove_raw(guard.closure_ptr);
                        guard.closure_ptr = ArenaPtr::null();
                    }
                }
            }
        }
    };
}

#[repr(C, align(16))]
pub struct FunctionAllocatorInner<const SIZE: usize> {
    data: UnsafeCell<[u8; SIZE]>,
    free_list: FreeList,
    mutex: Mutex<()>
}

impl<const SIZE: usize> FunctionAllocatorInner<SIZE> {
    pub const fn new() -> Self {
        Self {
            data: UnsafeCell::new([0; SIZE]),
            free_list: FreeList::new(),
            mutex: Mutex::new(()),
        }
    }

    fn find_space(&self, size: usize, align: usize) -> Option<FindSpaceResult> {
        unsafe {
            let data_ptr = self.data.get() as *mut u8;
            let header_size = mem::size_of::<ClosureHeader>();

            let mut offset = 0;
            let mut current = self.free_list.head();
            let mut prev = HeaderPtr::null();

            while !current.is_null() {
                let block_start = current.offset_from(data_ptr);
                let block_end = current.data_end(data_ptr);

                let data_start_candidate = offset + header_size;
                let data_aligned = (data_start_candidate + align - 1) & !(align - 1);
                let alloc_end = data_aligned + size;

                if alloc_end <= block_start {
                    break;
                }

                offset = block_end;
                prev = current;
                current = current.next();
            }

            let data_start_candidate = offset + header_size;
            let data_aligned_offset = (data_start_candidate + align - 1) & !(align - 1);
            let alloc_end_offset = data_aligned_offset + size;
            let header_offset = data_aligned_offset - header_size;

            if alloc_end_offset > SIZE {
                return None;
            }

            let mut check = self.free_list.head();
            while !check.is_null() {
                let check_start = check.offset_from(data_ptr);
                let check_data_end = check.data_end(data_ptr);
        
                if header_offset < check_data_end && alloc_end_offset > check_start {
                    return None;
                }
                check = HeaderPtr((*check.0).next);
            }

            let header_ptr = HeaderPtr(data_ptr.add(header_offset) as *mut ClosureHeader);
            let data_ptr = ArenaPtr(data_ptr.add(data_aligned_offset));

            Some(FindSpaceResult {
                header_offset,
                data_offset: data_aligned_offset,
                alloc_end: alloc_end_offset,
                header_ptr,
                data_ptr,
                prev,
                current,
            })
        }
    }

    fn insert_into_free_list(&self, header_ptr: HeaderPtr, prev: HeaderPtr, current: HeaderPtr) {
        unsafe {
            if prev.is_null() {
                self.free_list.set(header_ptr.0);
            } else {
                (*prev.0).next = header_ptr.0;
            }
        }
    }

    pub fn alloc(&self, size: usize, align: usize, block_type: BlockType) -> ArenaPtr {
        let _guard = self.mutex.lock();

        unsafe {
            let space = match self.find_space(size, align) {
                Some(s) => s,
                None => return ArenaPtr(ptr::null_mut()),
            };
            
            let header_size = mem::size_of::<ClosureHeader>();
            let total_size = header_size + size;
            space.header_ptr.write(ClosureMeta::new(size, align), space.current, block_type);

            self.insert_into_free_list(space.header_ptr, space.prev, space.current);

            space.data_ptr
        }
    }

    pub fn dealloc(&self, ptr: ArenaPtr) {
        let _guard = self.mutex.lock();

        unsafe {
            let header_ptr = HeaderPtr::from_data_ptr(ptr);
            header_ptr.try_drop_data();

            let mut current = self.free_list.head();
            let mut prev = HeaderPtr::null();

            while !current.is_null() && current.0 < header_ptr.0 {
                prev = current;
                current = current.next();
            }

            if current.0 == header_ptr.0 {
                return;
            }

            if !prev.is_null() && prev.0 == header_ptr.0 {
                return;
            }

            self.insert_into_free_list(header_ptr, prev, current);
        }
    }

    
    pub fn store_closure<F, T>(&self, closure: Closure<F, T>) -> ArenaPtr
    where
        F: FnOnce() -> T + Send + 'static,
    {
        unsafe {
            let size = closure.size();
            let align = closure.align();

            let allocated_ptr = self.alloc(size, align, BlockType::Closure);
            if allocated_ptr.is_null() {
                return ArenaPtr::null();
            }

            let data_ptr = allocated_ptr;
            closure.write(data_ptr);
            // ptr::copy_nonoverlapping(&closure as *const _ as *const u8, data_ptr.0, size);
            let header_ptr = HeaderPtr::from_data_ptr(data_ptr);
            header_ptr.set_meta::<F, T>(data_ptr, size, align);

            data_ptr
        }
    }

    pub fn alloc_packet<T>(&self, closure_ptr: ArenaPtr) -> *mut Packet<T> {
        unsafe {
            let packet_size = mem::size_of::<Packet<T>>();
            let packet_align = mem::align_of::<Packet<T>>();

            let allocated_ptr = self.alloc(packet_size, packet_align, BlockType::Packet);
            
            if allocated_ptr.is_null() {
                return ptr::null_mut();
            }

            let packet_ptr = allocated_ptr.as_packet_ptr();
            packet_ptr.write_to(closure_ptr);

            packet_ptr
        }
    }
        
    pub fn remove_packet<T>(&self, packet_ptr: *mut Packet<T>) {
        unsafe {
            if packet_ptr.is_null() {
                return;
            }
            let ptr = ArenaPtr(packet_ptr as *mut u8);
            ptr::drop_in_place(packet_ptr);

            let header_ptr = HeaderPtr::from_data_ptr(ptr);

            let mut current = self.free_list.head();
            let mut prev = HeaderPtr(ptr::null_mut());

            while !current.is_null() && current.0 < header_ptr.0 {
                prev = current;
                current = current.next();
            }

            if current.0 == header_ptr.0 {
                return;
            }

            if !prev.is_null() && prev.0 == header_ptr.0 {
                return;
            }

            self.insert_into_free_list(header_ptr, prev, current);
        }
    }

    pub fn remove_raw(&self, ptr: ArenaPtr) {
        unsafe {
            if ptr.is_null() {
                return;
            }
            self.dealloc(ptr);
        }
    }

    pub fn debug_dump(&self) {
        let _guard = self.mutex.lock();

        unsafe {
            println!("=== Arena Dump ===");
            println!("Total size: {} bytes", SIZE);
            let mut current = self.free_list.head();
            let mut idx = 0;
            let mut total_used = 0;
            let header_size = mem::size_of::<ClosureHeader>();
            
            while !current.is_null() {
                let header_offset = current.offset_from(self.data.get() as *mut u8);
                let data_offset = header_offset + header_size;
                let data_size = current.data_size();
                let data_align = current.data_align();
                let block_size = header_size + data_size;
                let block_end = header_offset + block_size;
                let block_type = current.block_type();
                
                println!("  Block #{}: type {:?}, {}-{} bytes ({} bytes total)", 
                    idx, block_type, header_offset, block_end, block_size);
                total_used += block_size;
                current = current.next();
                idx += 1;
            }
            
            println!("Free blocks: {}", idx);
            println!("Total used: {} bytes", total_used);
            println!("Free space: {} bytes", SIZE - total_used);
            println!("=== End Dump ===");
        }
    }

    pub fn clear(&self) {
        unsafe {
            let mut current = self.free_list.head();
            while !current.is_null() {
                current.try_drop_data();
                current = current.next();
            }
            self.free_list.set(ptr::null_mut());
        }
    }

    pub fn count(&self) -> usize {
        unsafe {
            let mut count = 0;
            let mut current = self.free_list.head();
            while !current.is_null() {
                count += 1;
                current = current.next();
            }
            count
        }
    }

    pub fn blocks(&self) -> BlocksIter {
        unsafe {
            BlocksIter::new(self.free_list.get(), self.data.get() as *mut u8)
        }
    }
}

pub struct BlocksIter {
    current: *mut ClosureHeader,
    data_ptr: *mut u8,
}

impl BlocksIter {
    pub unsafe fn new(current: *mut ClosureHeader, data_ptr: *mut u8) -> Self {
        Self { current, data_ptr }
    }
}

impl Iterator for BlocksIter {
    type Item = BlockInfo;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.current.is_null() {
                return None;
            }
            let header = &*self.current;
            let header_offset = (self.current as usize) - (self.data_ptr as usize);
            let data_offset = header_offset + mem::size_of::<ClosureHeader>();
            let info = BlockInfo {
                header_offset,
                data_offset,
                data_size: header.meta.data_size,
                data_align: header.meta.data_align,
            };
            self.current = header.next;
            Some(info)
        }
    }
}

#[derive(Clone, Copy)]
pub struct ClosureHeader {
    meta: ClosureMeta,
    next: *mut ClosureHeader,
    block_type: BlockType,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlockType {
    Free,
    Closure,
    Packet,
}

#[derive(Clone, Copy)]
pub struct ClosureMeta {
    func_ptr: FuncPtr,
    data_ptr: DataPtr,
    data_size: usize,
    data_align: usize,
    type_id: TypeId,
    drop_fn: DropFn,
}

impl ClosureMeta {
    pub fn new(size: usize, align: usize) -> Self {
        Self {
            func_ptr: FuncPtr(ptr::null()),
            data_ptr: DataPtr(ptr::null_mut()),
            data_size: size,
            data_align: align,
            type_id: TypeId::of::<()>(),
            drop_fn: DropFn(drop_nothing),
        }
    }
}

#[derive(Clone, Copy)]
pub struct FuncPtr(*const u8);

#[derive(Clone, Copy)]
pub struct DataPtr(*mut u8);

#[derive(Clone, Copy)]
pub struct ArenaPtr(pub *mut u8);

impl ArenaPtr {
    pub fn as_packet_ptr<T>(&self) -> *mut Packet<T> {
        self.0 as *mut Packet<T>
    }

    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }

    pub fn null() -> Self {
        Self(ptr::null_mut())
    }
}

impl Default for ArenaPtr {
    fn default() -> Self {
        Self::null()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct HeaderPtr(*mut ClosureHeader);

impl HeaderPtr {
    pub fn next(&self) -> Self {
        unsafe {
            Self((*self.0).next)
        }
    }

    pub fn null() -> Self {
        Self(ptr::null_mut())
    }

    pub fn data_size(&self) -> usize {
        unsafe { (*self.0).meta.data_size }
    }
    
    pub fn data_align(&self) -> usize {
        unsafe { (*self.0).meta.data_align }
    }

    pub fn block_type(&self) -> BlockType {
        unsafe { (*self.0).block_type }
    }

    pub fn from_data_ptr(data_ptr: ArenaPtr) -> Self {
        unsafe {
            Self((data_ptr.0 as usize - mem::size_of::<ClosureHeader>()) as *mut ClosureHeader)
        }
    }

    pub fn try_drop_data(&self) {
        unsafe {
            let size = (*self.0).meta.data_size;
            if size > 0 {
                let drop_fn = (*self.0).meta.drop_fn;
                let data_ptr = (*self.0).meta.data_ptr;
                (drop_fn.0)(data_ptr.0);
            }
        }
    }

    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }

    pub fn offset_from(&self, base: *mut u8) -> usize {
        (self.0 as usize) - (base as usize)
    }

    pub fn write(&self, meta: ClosureMeta, next: HeaderPtr, block_type: BlockType) {
        unsafe {
            ptr::write(
                self.0,
                ClosureHeader {
                    meta,
                    next: next.0,
                    block_type,
                },
            );
        }
    }

    pub fn set_meta<F, T>(&self, data_ptr: ArenaPtr, size: usize, align: usize)
    where
        F: FnOnce() -> T + 'static,
    {
        unsafe {
            (*self.0).meta = ClosureMeta {
                func_ptr: FuncPtr(call_closure::<F, T> as *const () as *const u8),
                data_ptr: DataPtr(data_ptr.0),
                data_size: size,
                data_align: align,
                type_id: TypeId::of::<F>(),
                drop_fn: DropFn(drop_closure::<F>),
            };
        }
    }

    pub fn data_end(&self, base: *mut u8) -> usize {
        unsafe {
            let header_size = mem::size_of::<ClosureHeader>();
            let block_start = self.offset_from(base);
            let data_start = block_start + header_size;
            data_start + (*self.0).meta.data_size
        }
    }
}

pub struct FindSpaceResult {
    pub header_offset: usize,
    pub data_offset: usize,
    pub alloc_end: usize,
    pub header_ptr: HeaderPtr,
    pub data_ptr: ArenaPtr,
    pub prev: HeaderPtr,
    pub current: HeaderPtr,
}

#[derive(Clone, Copy)]
pub struct DropFn(unsafe fn(*mut u8));

pub struct BlockInfo {
    pub header_offset: usize,
    pub data_offset: usize,
    pub data_size: usize,
    pub data_align: usize,
}

fn drop_nothing(_ptr: *mut u8) {}

fn call_closure<F, T>(data_ptr: *mut u8) -> T
where
    F: FnOnce() -> T,
{
    unsafe {
        let closure: F = ptr::read(data_ptr as *const F);
        closure()
    }
}

fn drop_closure<F>(data_ptr: *mut u8) {
    unsafe {
        ptr::drop_in_place(data_ptr as *mut F);
    }
}

pub struct FreeList(UnsafeCell<*mut ClosureHeader>);

impl FreeList {
    pub const fn new() -> Self {
        Self(UnsafeCell::new(ptr::null_mut()))
    }

    pub unsafe fn get(&self) -> *mut ClosureHeader {
        *self.0.get()
    }

    pub unsafe fn set(&self, ptr: *mut ClosureHeader) {
        *self.0.get() = ptr;
    }

    pub unsafe fn head(&self) -> HeaderPtr {
        HeaderPtr(*self.0.get())
    }
}

pub struct Closure<F: FnOnce() -> T + Send + 'static, T>(F);

impl<F: FnOnce() -> T + Send + 'static, T> Closure<F, T> {
    pub const fn new(func: F) -> Self {
        Self(func)
    }

    pub const fn size(&self) -> usize {
        mem::size_of_val(&self)
    }

    pub const fn align(&self) -> usize {
        mem::align_of_val(&self)
    }

    pub fn write(&self, data_ptr: ArenaPtr) {
        unsafe { ptr::copy_nonoverlapping(&self as *const _ as *const u8, data_ptr.0, self.size()) }
    }
}