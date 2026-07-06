#![warn(unsafe_op_in_unsafe_fn)]
#![warn(clippy::undocumented_unsafe_blocks)]

use core::alloc::{GlobalAlloc, Layout};
use core::mem::MaybeUninit;
use core::ptr;

use crate::custalloc::buffer::HEADER_SIZE;
use crate::custalloc::entry::{Entry, State};
use crate::mutex::Mutex;

pub struct CustAllocator<const N: usize>(Mutex<RawAllocator<N>>);

impl<const N: usize> CustAllocator<N> {
    pub const fn new() -> Self {
        let raw = Mutex::new(RawAllocator::new());
        Self(raw)
    }

    unsafe fn align_to(ptr: *mut u8, align: usize) -> *mut u8 {
        let addr = ptr as usize;
        let mismatch = addr & (align - 1);
        let offset = if mismatch == 0 { 0 } else { align - mismatch };
        unsafe { ptr.add(offset) }
    }
}

unsafe impl<const N: usize> GlobalAlloc for CustAllocator<N> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = if align > 4 {
            layout.size() + align
        } else {
            layout.size()
        };

        self.0
            .lock()
            .alloc(size)
            .map_or(ptr::null_mut(), |memory| {
                unsafe { Self::align_to(ptr::addr_of_mut!(*memory).cast(), align) }
            })
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        let _maybe_error = self.0.lock().free(ptr.cast()).ok();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FreeError {
    DoubleFreeDetected,
    AllocationNotFound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidatedOffset(usize);

pub struct RawAllocator<const N: usize>(buffer::Buffer<N>);

impl<const N: usize> RawAllocator<N> {
    pub const fn new() -> Self {
        assert!(N >= 8, "too small heap memory: minimum size is 8");
        assert!(N % 4 == 0, "memory size has to be divisible by 4");

        let buffer = buffer::Buffer::new();
        Self(buffer)
    }

    pub fn alloc(&mut self, n: usize) -> Option<&mut [MaybeUninit<u8>]> {
        self.0.ensure_initialization();

        let n = (n + HEADER_SIZE - 1) / HEADER_SIZE * HEADER_SIZE;

        let (offset, _) = self
            .0
            .entries()
            .map(|offset| (offset, self.0[offset]))
            .filter(|(_offset, entry)| entry.state() == State::Free)
            .filter(|(_offset, entry)| entry.size() >= n)
            .min_by_key(|(_offset, entry)| entry.size())?;

        self.0.mark_as_used(offset, n);
        Some(self.0.memory_of_mut(offset))
    }

    pub fn free(&mut self, ptr: *mut u8) -> Result<(), FreeError> {
        self.0.ensure_initialization();

        let offset = self
            .0
            .entries()
            .find(|offset| {
                let size = self.0[*offset].size();
                let memory = self.0.memory_of(*offset);
                let ptr = ptr as *const _;
                let start = memory.as_ptr();
                let end = start.wrapping_add(size);

                start <= ptr && ptr < end
            })
            .ok_or(FreeError::AllocationNotFound)?;

        let entry = self.0[offset];
        if entry.state() == State::Free {
            return Err(FreeError::DoubleFreeDetected);
        }

        let additional_memory = self
            .0
            .following_free_entry(offset)
            .map_or(0, |entry| entry.size() + HEADER_SIZE);

        self.0[offset] = Entry::free(entry.size() + additional_memory);
        Ok(())
    }
}

mod buffer {
    use super::entry::{Entry, State};
    use core::mem::MaybeUninit;

    pub const HEADER_SIZE: usize = 4;

    #[repr(align(4))]
    pub struct Buffer<const N: usize>([MaybeUninit<u8>; N]);

    impl<const N: usize> Buffer<N> {
        pub const fn new() -> Self {
            let mut buffer = [MaybeUninit::uninit(); N];
            buffer[0] = MaybeUninit::new(0x00);
            buffer[1] = MaybeUninit::new(0x00);
            buffer[2] = MaybeUninit::new(0x00);
            buffer[3] = MaybeUninit::new(0x00);
            Self(buffer)
        }

        pub fn ensure_initialization(&mut self) {
            let not_yet_initialized = self.0[0..4]
                .iter()
                .map(|byte| unsafe { byte.assume_init() })
                .all(|byte| byte == 0x00);

            if not_yet_initialized {
                let initial_entry = Entry::free(N - HEADER_SIZE).as_raw();
                self.0[0] = MaybeUninit::new(initial_entry[0]);
                self.0[1] = MaybeUninit::new(initial_entry[1]);
                self.0[2] = MaybeUninit::new(initial_entry[2]);
                self.0[3] = MaybeUninit::new(initial_entry[3]);
            }
        }

        pub fn entries(&self) -> impl Iterator<Item = usize> + '_ {
            let mut offset = 0;
            core::iter::from_fn(move || {
                if offset >= N {
                    return None;
                }

                let current = offset;
                let entry = self[current];
                offset += entry.size() + HEADER_SIZE;
                Some(current)
            })
        }

        pub fn following_free_entry(&self, offset: usize) -> Option<Entry> {
            let entry = self[offset];
            if entry.state() == State::Free {
                return Some(entry);
            }

            let next_offset = offset + entry.size() + HEADER_SIZE;
            if next_offset >= N {
                return None;
            }

            let next_entry = self[next_offset];
            if next_entry.state() == State::Free {
                Some(next_entry)
            } else {
                None
            }
        }

        pub fn mark_as_used(&mut self, offset: usize, size: usize) {
            let entry = self[offset];
            let remaining = entry.size() - size;

            self[offset] = Entry::used(size);

            if remaining >= HEADER_SIZE {
                let next_offset = offset + size + HEADER_SIZE;
                self[next_offset] = Entry::free(remaining - HEADER_SIZE);
            }
        }

        pub fn memory_of(&self, offset: usize) -> &[MaybeUninit<u8>] {
            let entry = self[offset];
            let start = offset + HEADER_SIZE;
            let end = start + entry.size();
            &self.0[start..end]
        }

        pub fn memory_of_mut(&mut self, offset: usize) -> &mut [MaybeUninit<u8>] {
            let entry = self[offset];
            let start = offset + HEADER_SIZE;
            let end = start + entry.size();
            &mut self.0[start..end]
        }
    }

    impl<const N: usize> core::ops::Index<usize> for Buffer<N> {
        type Output = Entry;

        fn index(&self, index: usize) -> &Self::Output {
            unsafe {
                let ptr = self.0.as_ptr().add(index) as *const Entry;
                &*ptr
            }
        }
    }

    impl<const N: usize> core::ops::IndexMut<usize> for Buffer<N> {
        fn index_mut(&mut self, index: usize) -> &mut Self::Output {
            unsafe {
                let ptr = self.0.as_mut_ptr().add(index) as *mut Entry;
                &mut *ptr
            }
        }
    }
}

mod entry {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Entry(u32);

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum State {
        Free,
        Used,
    }

    impl Entry {
        pub const fn used(size: usize) -> Self {
            Self(size as u32)
        }

        pub const fn free(size: usize) -> Self {
            Self((size as u32) | 1)
        }

        pub fn state(&self) -> State {
            if self.0 & 1 == 1 {
                State::Free
            } else {
                State::Used
            }
        }

        pub fn size(&self) -> usize {
            (self.0 & !1) as usize
        }

        pub const fn as_raw(self) -> [u8; 4] {
            self.0.to_ne_bytes()
        }
    }
}