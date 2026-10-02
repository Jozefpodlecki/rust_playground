use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::mem;
use core::ops::{Deref, DerefMut};
use core::ptr;
use core::sync::atomic::{AtomicU8, Ordering};

const HEADER_SIZE: usize = 2 * mem::size_of::<usize>();

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MutexState {
    Unlocked = 0,
    Locked = 1,
    Contended = 2,
}

#[repr(transparent)]
pub(crate) struct AtomicMutexState(AtomicU8);

impl AtomicMutexState {
    pub const fn new(state: MutexState) -> Self {
        Self(AtomicU8::new(state as u8))
    }

    pub const fn load(&self) -> MutexState {
        match self.0.load(Ordering::Acquire) {
            0 => MutexState::Unlocked,
            1 => MutexState::Locked,
            2 => MutexState::Contended,
            _ => MutexState::Unlocked,
        }
    }

    pub const fn try_acquire(&self) -> bool {
        self.0
            .compare_exchange_weak(
                MutexState::Unlocked as u8,
                MutexState::Locked as u8,
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            .is_ok()
    }

    pub const fn release(&self) {
        self.0.store(MutexState::Unlocked as u8, Ordering::Release);
    }
}

#[repr(C)]
struct FreeNode {
    next: *mut FreeNode,
    size: usize,
}

impl FreeNode {
    unsafe fn set_next(self: *mut Self, value: *mut FreeNode) {
        unsafe { (*self).next = value };
    }
    unsafe fn next(self: *const Self) -> *mut FreeNode {
        unsafe { (*self).next }
    }
    unsafe fn set_size(self: *mut Self, value: usize) {
        unsafe { (*self).size = value };
    }
    unsafe fn size(self: *const Self) -> usize {
        unsafe { (*self).size }
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

pub struct FreeListAllocator<const N: usize> {
    state: AtomicMutexState,
    inner: UnsafeCell<FreeListAllocatorInner<N>>,
}

pub struct FreeListAllocatorInner<const N: usize> {
    buffer: AllocatorBuffer<N>,
    head: *mut FreeNode,
    initialized: bool,
}

impl<const N: usize> FreeListAllocator<N> {
    pub const fn new() -> Self {
        Self {
            state: AtomicMutexState::new(MutexState::Unlocked),
            inner: UnsafeCell::new(FreeListAllocatorInner {
                buffer: AllocatorBuffer::new(),
                head: ptr::null_mut(),
                initialized: false,
            }),
        }
    }

    #[inline(always)]
    fn lock(&self) -> AllocatorGuard<'_, N> {
        while !self.state.try_acquire() {
            while self.state.load() != MutexState::Unlocked {
                core::hint::spin_loop();
            }
        }
        let inner = unsafe { &mut *self.inner.get() };
        AllocatorGuard { inner, state: &self.state }
    }
}

pub struct AllocatorGuard<'a, const N: usize> {
    inner: &'a mut FreeListAllocatorInner<N>,
    state: &'a AtomicMutexState,
}

impl<'a, const N: usize> Drop for AllocatorGuard<'a, N> {
    fn drop(&mut self) {
        self.state.release();
    }
}

impl<'a, const N: usize> Deref for AllocatorGuard<'a, N> {
    type Target = FreeListAllocatorInner<N>;
    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, const N: usize> DerefMut for AllocatorGuard<'a, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
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

        unsafe {
            buffer_ptr.set_next(ptr::null_mut());
            buffer_ptr.set_size(total_size);
        }
        self.head = buffer_ptr;
    }

    #[inline(always)]
    fn align_size(&self, size: usize) -> usize {
        (size + mem::size_of::<usize>() - 1) & !(mem::size_of::<usize>() - 1)
    }

    #[inline(always)]
    unsafe fn block_size(&self, node: *mut FreeNode) -> usize {
        if node.is_null() {
            0
        } else {
            unsafe { node.size() }
        }
    }

    #[inline(always)]
    unsafe fn can_split(&self, node: *mut FreeNode, needed: usize) -> bool {
        unsafe { self.block_size(node) >= needed + mem::size_of::<FreeNode>() }
    }

    #[inline(always)]
    unsafe fn split(&mut self, node: *mut FreeNode, needed: usize) {
        unsafe {
            let curr_addr = node as usize;
            let old_size = node.size();
            let old_next = node.next();

            let new_free = (curr_addr + needed) as *mut FreeNode;
            new_free.set_next(old_next);
            new_free.set_size(old_size - needed);
            node.set_next(new_free);
            node.set_size(needed);
        }
    }

    #[inline(always)]
    unsafe fn remove(&mut self, prev: *mut FreeNode, curr: *mut FreeNode) {
        unsafe {
            let next = curr.next();
            if prev.is_null() {
                self.head = next;
            } else {
                prev.set_next(next);
            }
        }
    }

    fn find_fit(&mut self, size_aligned: usize, align: usize) -> (*mut FreeNode, usize) {
        let mut prev = ptr::null_mut();
        let mut curr = self.head;

        while !curr.is_null() {
            let curr_addr = curr as usize;
            let padding = (align - ((curr_addr + HEADER_SIZE) % align)) % align;
            let total_needed = padding + HEADER_SIZE + size_aligned;

            unsafe {
                if self.block_size(curr) >= total_needed {
                    if self.can_split(curr, total_needed) {
                        self.split(curr, total_needed);
                    }
                    self.remove(prev, curr);
                    return (curr, padding);
                }
                prev = curr;
                curr = curr.next();
            }
        }

        (ptr::null_mut(), 0)
    }

    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        self.init();

        let align = layout.align().max(mem::size_of::<usize>());
        let size_aligned = self.align_size(layout.size());

        let (node, padding) = self.find_fit(size_aligned, align);
        if node.is_null() {
            return ptr::null_mut();
        }

        unsafe {
            let block_size = self.block_size(node);
            let node_addr = node as usize;
            let user_ptr = node_addr + padding + HEADER_SIZE;

            let header = (user_ptr - HEADER_SIZE) as *mut usize;
            *header = padding;
            *header.add(1) = block_size;

            user_ptr as *mut u8
        }
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8) {
        if ptr.is_null() {
            return;
        }

        unsafe {
            let ptr_addr = ptr as usize;
            let header = (ptr_addr - HEADER_SIZE) as *mut usize;
            let padding = *header;
            let total_size = *header.add(1);

            let block_start_usize = ptr_addr - HEADER_SIZE - padding;

            let start_addr = self.buffer.start_addr();
            let end_addr = self.buffer.end_addr();

            if block_start_usize < start_addr || block_start_usize >= end_addr {
                return;
            }
            if total_size == 0 || total_size > (end_addr - start_addr) {
                return;
            }
            if block_start_usize + total_size > end_addr {
                return;
            }

            let node = block_start_usize as *mut FreeNode;

            let mut cur = self.head;
            while !cur.is_null() {
                if cur == node {
                    return;
                }
                cur = cur.next();
            }

            let mut prev = ptr::null_mut();
            let mut cur = self.head;
            let node_usize = node as usize;
            while !cur.is_null() && (cur as usize) < node_usize {
                prev = cur;
                cur = cur.next();
            }

            node.set_size(total_size);
            node.set_next(cur);
            if prev.is_null() {
                self.head = node;
            } else {
                prev.set_next(node);
            }

            let next = node.next();
            if !next.is_null() {
                let node_end = node_usize + node.size();
                if node_end == (next as usize) {
                    node.set_size(node.size() + next.size());
                    node.set_next(next.next());
                }
            }

            if !prev.is_null() {
                let prev_end = (prev as usize) + prev.size();
                if prev_end == node_usize {
                    prev.set_size(prev.size() + node.size());
                    prev.set_next(node.next());
                }
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
        unsafe { guard.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        if ptr.is_null() {
            return;
        }
        let mut guard = self.lock();
        unsafe { guard.dealloc(ptr) }
    }
}

unsafe impl<const N: usize> Sync for FreeListAllocator<N> {}

#[cfg(test)]
mod tests {
    use super::*;
    use core::alloc::{GlobalAlloc, Layout};

    extern crate alloc;

    const CAP: usize = 4096;

    #[test]
    fn alloc_dealloc_roundtrip() {
        let a = FreeListAllocator::<CAP>::new();
        let layout = Layout::from_size_align(64, 8).unwrap();
        let p = unsafe { a.alloc(layout) };
        assert!(!p.is_null());
        unsafe {
            for i in 0..64u8 {
                *p.add(i as usize) = i;
            }
            for i in 0..64u8 {
                assert_eq!(*p.add(i as usize), i);
            }
            a.dealloc(p, layout);
        }
    }

    #[test]
    fn respects_alignment() {
        let a = FreeListAllocator::<CAP>::new();
        for align in [1usize, 2, 4, 8, 16, 64] {
            let layout = Layout::from_size_align(32, align).unwrap();
            let p = unsafe { a.alloc(layout) };
            assert!(!p.is_null());
            assert_eq!(p as usize % align, 0);
            unsafe { a.dealloc(p, layout) };
        }
    }

    #[test]
    fn reuses_freed_block() {
        let a = FreeListAllocator::<CAP>::new();
        let layout = Layout::from_size_align(32, 8).unwrap();
        let p1 = unsafe { a.alloc(layout) };
        assert!(!p1.is_null());
        unsafe { a.dealloc(p1, layout) };
        let p2 = unsafe { a.alloc(layout) };
        assert_eq!(p1, p2);
        unsafe { a.dealloc(p2, layout) };
    }

    #[test]
    fn coalesces_adjacent_frees() {
        let a = FreeListAllocator::<CAP>::new();
        let layout = Layout::from_size_align(64, 8).unwrap();
        let p1 = unsafe { a.alloc(layout) };
        let p2 = unsafe { a.alloc(layout) };
        let p3 = unsafe { a.alloc(layout) };
        assert!(!p1.is_null() && !p2.is_null() && !p3.is_null());
        unsafe {
            a.dealloc(p1, layout);
            a.dealloc(p2, layout);
            a.dealloc(p3, layout);
        }
        let big = Layout::from_size_align(192, 8).unwrap();
        let p = unsafe { a.alloc(big) };
        assert!(!p.is_null());
        unsafe { a.dealloc(p, big) };
    }

    #[test]
    fn exhaustion_and_recovery() {
        let a = FreeListAllocator::<CAP>::new();
        let layout = Layout::from_size_align(256, 8).unwrap();
        let mut ptrs = alloc::vec::Vec::new();
        loop {
            let p = unsafe { a.alloc(layout) };
            if p.is_null() {
                break;
            }
            ptrs.push(p);
        }
        assert!(!ptrs.is_empty());
        for &p in &ptrs {
            unsafe { a.dealloc(p, layout) };
        }
        ptrs.clear();
        let p = unsafe { a.alloc(layout) };
        assert!(!p.is_null());
        unsafe { a.dealloc(p, layout) };
    }
}