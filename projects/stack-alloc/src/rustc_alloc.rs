use ntapi::ntpsapi::NtCurrentProcess;
use toolkit::{ProcessEnvironmentBlock, ProcessMemoryProtector, ProcessMemoryReader, ProcessMemoryWriter, };
use winapi::um::winnt::{IMAGE_DOS_HEADER, IMAGE_NT_HEADERS64};

#[macro_export]
macro_rules! init_arena {
    ($size:expr) => {{
        pub type FreeListAllocator = crate::allocator::FreeListAllocator::<$size>;

        impl FreeListAllocator {

            pub fn get() -> Self {
                Self(crate::rustc_alloc::read_arena_ptr())
            }
        }

        #[rustc_std_internal_symbol]
        #[rustc_allocator]
        pub fn __rust_alloc(size: usize, align: ::core::mem::Alignment) -> *mut u8 {
            let allocator = FreeListAllocator::get();

            unsafe { ::core::alloc::GlobalAlloc::alloc(
                &allocator,
                ::core::alloc::Layout::from_size_alignment_unchecked(size, align),
            ) }
        }

        #[rustc_std_internal_symbol]
        #[rustc_deallocator]
        pub fn __rust_dealloc(
            ptr: *mut u8,
            size: usize,
            align: ::core::mem::Alignment,
        ) -> () {
            let allocator = FreeListAllocator::get();

            unsafe { ::core::alloc::GlobalAlloc::dealloc(
                &allocator,
                ptr,
                ::core::alloc::Layout::from_size_alignment_unchecked(size, align),
            ) }
        }

        #[rustc_std_internal_symbol]
        #[rustc_reallocator]
        pub fn __rust_realloc(
            ptr: *mut u8,
            size: usize,
            align: ::core::mem::Alignment,
            new_size: usize,
        ) -> *mut u8 {
            let allocator = FreeListAllocator::get();

            unsafe { ::core::alloc::GlobalAlloc::realloc(
                &allocator,
                ptr,
                ::core::alloc::Layout::from_size_alignment_unchecked(size, align),
                new_size,
            ) }
        }

        #[rustc_std_internal_symbol]
        #[rustc_allocator_zeroed]
        pub fn __rust_alloc_zeroed(
            size: usize,
            align: ::core::mem::Alignment,
        ) -> *mut u8 {
            let allocator = FreeListAllocator::get();

            unsafe {::core::alloc::GlobalAlloc::alloc_zeroed(
                &allocator,
                ::core::alloc::Layout::from_size_alignment_unchecked(size, align),
            ) }
        }

        unsafe {
            core::arch::asm!("nop dword ptr [rax+rax*1+0x00000000]");
        }
        
        let mut arena: [u8; $size] = unsafe { core::mem::zeroed() };
        crate::rustc_alloc::write_arena_to_entrypoint(&mut arena);

        arena
    }};
}

pub fn write_arena_to_entrypoint<T>(arena: &mut T) {
    let protector = ProcessMemoryProtector::current();
    let arena_ptr = core::ptr::addr_of_mut!(*arena) as *mut u8;
    let addr = get_entrypoint_address() as _;

    protector.make_writable(addr, 1).unwrap();
    let ptr_bytes = (arena_ptr as usize).to_ne_bytes();
    let slice = ptr_bytes.as_ref();
    ProcessMemoryWriter::write(addr as _, slice).unwrap();
}

pub fn get_entrypoint_address() -> *mut u8 {
    let image_base = ProcessEnvironmentBlock::current_process().image_base();
    let handle = NtCurrentProcess;
    let dos_header: IMAGE_DOS_HEADER = ProcessMemoryReader::read_remote(handle, image_base as _).unwrap();
    let nt_headers_addr = image_base as usize + dos_header.e_lfanew as usize;
    let nt_headers: IMAGE_NT_HEADERS64 = ProcessMemoryReader::read_remote(handle, nt_headers_addr as _).unwrap();
    let entrypoint_rva = nt_headers.OptionalHeader.AddressOfEntryPoint;
    (image_base as usize + entrypoint_rva as usize) as *mut u8
}

pub fn read_arena_ptr() -> *mut u8 {
    let entrypoint_addr = get_entrypoint_address() as *const *mut u8;
    unsafe { *entrypoint_addr }
}