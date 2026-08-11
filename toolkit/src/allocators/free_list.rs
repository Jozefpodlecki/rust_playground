use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::mem;
use core::ops::{Deref, DerefMut};
use core::ptr;
use core::sync::atomic::Ordering;

use crate::{AtomicMutexState, MutexState};
use crate::futex::{wait_on_address, wake_by_address_single};

const HEADER_SIZE: usize = mem::size_of::<usize>();

#[repr(C)]
struct FreeNode {
    next: *mut FreeNode,
}

impl FreeNode {
    #[inline(always)]
    fn set_next(self: *mut Self, value: *mut FreeNode) {
        unsafe { (*self).next = value; }
    }
    #[inline(always)]
    fn next(self: *const Self) -> *mut FreeNode {
        unsafe { (*self).next }
    }
}

#[repr(align(8))]
pub struct AllocatorBuffer<const N: usize>([u8; N]);

impl<const N: usize> AllocatorBuffer<N> {
    const fn new() -> Self {
        Self([0; N])
    }
    #[inline(always)]
    fn as_freenode_ptr(&mut self) -> *mut FreeNode {
        self.0.as_mut_ptr() as *mut FreeNode
    }
    #[inline(always)]
    fn start_addr(&self) -> usize {
        self.0.as_ptr() as usize
    }
    #[inline(always)]
    fn end_addr(&self) -> usize {
        self.0.as_ptr() as usize + N
    }
}

pub struct FreeListAllocator<const N: usize>(UnsafeCell<FreeListAllocatorInner<N>>);

pub type FreeListAllocator1KB = FreeListAllocator<{1024}>;
pub type FreeListAllocator1MB = FreeListAllocator<{1024 * 1024}>;
pub type FreeListAllocator1GB = FreeListAllocator<{1024 * 1024 * 1024}>;

pub struct FreeListAllocatorInner<const N: usize> {
    state: AtomicMutexState,
    buffer: AllocatorBuffer<N>,
    head: *mut FreeNode,
    initialized: bool,
}

impl<const N: usize> FreeListAllocator<N> {
    pub const fn new() -> Self {
        Self(UnsafeCell::new(FreeListAllocatorInner {
            state: AtomicMutexState::new(MutexState::Unlocked),
            buffer: AllocatorBuffer::new(),
            head: ptr::null_mut(),
            initialized: false,
        }))
    }

    #[inline(always)]
    fn lock(&self) -> AllocatorGuard<'_, N> {
        let inner = unsafe { &mut *self.0.get() };
        if !inner.state.try_acquire() {
            inner.lock_contended();
        }
        AllocatorGuard(inner)
    }
}

pub struct AllocatorGuard<'a, const N: usize>(&'a mut FreeListAllocatorInner<N>);

impl<'a, const N: usize> Drop for AllocatorGuard<'a, N> {
    fn drop(&mut self) {
        unsafe { self.0.unlock(); }
    }
}

impl<'a, const N: usize> Deref for AllocatorGuard<'a, N> {
    type Target = FreeListAllocatorInner<N>;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a, const N: usize> DerefMut for AllocatorGuard<'a, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl<const N: usize> FreeListAllocatorInner<N> {
    fn init(&mut self) {
        if self.initialized {
            return;
        }
        self.initialized = true;
        let buffer_ptr = self.buffer.as_freenode_ptr();
        let total_size = self.buffer.end_addr() - self.buffer.start_addr();
        if total_size < mem::size_of::<FreeNode>() {
            return;
        }
        self.head = buffer_ptr;
        unsafe { self.head.set_next(ptr::null_mut()); }
    }

    #[inline(always)]
    fn spin(&self) -> MutexState {
        let mut spin = 100;
        loop {
            let state = self.state.load();
            if state != MutexState::Locked || spin == 0 {
                return state;
            }
            core::hint::spin_loop();
            spin -= 1;
        }
    }

    #[cold]
    fn lock_contended(&self) {
        let mut state = self.spin();
        if state == MutexState::Unlocked {
            if self.state.try_acquire() {
                return;
            }
            state = self.state.load();
        }
        loop {
            if state != MutexState::Contended && self.state.mark_contended() {
                return;
            }
            unsafe {
                wait_on_address(&self.state.0, MutexState::Contended as u8, None);
            }
            state = self.spin();
        }
    }

    #[inline(always)]
    unsafe fn unlock(&self) {
        if self.state.unlock() {
            unsafe { wake_by_address_single(&self.state.0); }
        }
    }

    #[inline(always)]
    fn align_size(&self, size: usize) -> usize {
        (size + mem::size_of::<usize>() - 1) & !(mem::size_of::<usize>() - 1)
    }

    #[inline(always)]
    fn block_size(&self, node: *mut FreeNode) -> usize {
        let curr_addr = node as usize;
        let next_addr = unsafe { node.next() as usize };
        let end = if next_addr == 0 {
            self.buffer.end_addr()
        } else {
            next_addr
        };
        if end <= curr_addr {
            return 0;
        }
        end - curr_addr
    }

    #[inline(always)]
    fn can_split(&self, node: *mut FreeNode, needed: usize) -> bool {
        let size = self.block_size(node);
        size > needed + mem::size_of::<FreeNode>()
    }

    #[inline(always)]
    fn split(&mut self, node: *mut FreeNode, needed: usize) {
        let curr_addr = node as usize;
        let next = unsafe { node.next() };
        let new_free = (curr_addr + needed) as *mut FreeNode;
        unsafe {
            new_free.set_next(next);
            node.set_next(new_free);
        }
    }

    #[inline(always)]
    fn remove(&mut self, prev: &mut *mut FreeNode, curr: *mut FreeNode) -> *mut u8 {
        unsafe {
            let next = curr.next();
            if !prev.is_null() {
                (*prev).set_next(next);
            } else {
                self.head = next;
            }
            curr as *mut u8
        }
    }

    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        self.init();

        let size_aligned = self.align_size(layout.size());
        let align = layout.align();

        let mut prev = ptr::null_mut();
        let mut curr = self.head;

        while !curr.is_null() {
            let curr_addr = curr as usize;
            let user_start = curr_addr + HEADER_SIZE;
            let padding = (align - (user_start % align)) % align;
            let total_needed = HEADER_SIZE + padding + size_aligned;
            let block_size = self.block_size(curr);

            if block_size >= total_needed {
                if padding > 0 {
                    let new_block = (curr_addr + padding) as *mut FreeNode;
                    curr.set_next(new_block);
                    curr = new_block;
                }

                if self.can_split(curr, total_needed) {
                    self.split(curr, total_needed);
                }

                let result = self.remove(&mut prev, curr);
                *(curr as *mut usize) = total_needed;
                return result.add(HEADER_SIZE + padding);
            }

            prev = curr;
            curr = unsafe { curr.next() };
        }

        ptr::null_mut()
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8) {
        if ptr.is_null() {
            return;
        }

        let block_start = ptr.sub(HEADER_SIZE);
        let start_addr = self.buffer.start_addr();
        let end_addr = self.buffer.end_addr();

        let block_start_usize = block_start as usize;
        if block_start_usize < start_addr || block_start_usize >= end_addr {
            return;
        }

        let total_size = *(block_start as *mut usize);
        if total_size == 0 || total_size > (end_addr - start_addr) {
            return;
        }

        let block_end = block_start_usize + total_size;
        if block_end > end_addr {
            return;
        }

        let node = block_start as *mut FreeNode;

        let mut cur = self.head;
        while !cur.is_null() {
            if cur == node {
                return;
            }
            cur = unsafe { cur.next() };
        }

        let mut prev = ptr::null_mut();
        let mut cur = self.head;
        let node_usize = node as usize;

        while !cur.is_null() && (cur as usize) < node_usize {
            prev = cur;
            cur = unsafe { cur.next() };
        }

        unsafe { node.set_next(cur); }
        if !prev.is_null() {
            unsafe { prev.set_next(node); }
        } else {
            self.head = node;
        }

        let next = unsafe { node.next() };
        if !next.is_null() {
            let node_end = node_usize + total_size;
            if node_end == (next as usize) {
                unsafe { node.set_next(next.next()); }
            }
        }

        if !prev.is_null() {
            let prev_end = (prev as usize) + self.block_size(prev);
            if prev_end == node_usize {
                unsafe { prev.set_next(node.next()); }
            }
        }
    }
}

unsafe impl<const N: usize> GlobalAlloc for FreeListAllocator<N> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.size() == 0 {
            return ptr::null_mut();
        }
        let mut guard = self.lock();
        guard.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        if ptr.is_null() {
            return;
        }
        let mut guard = self.lock();
        guard.dealloc(ptr);
    }
}

unsafe impl<const N: usize> Sync for FreeListAllocator<N> {}