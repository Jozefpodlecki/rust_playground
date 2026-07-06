use core::{cell::UnsafeCell, marker::PhantomData, mem::MaybeUninit, sync::atomic::{Atomic, AtomicBool, AtomicUsize}};
use core::sync::{atomic::{AtomicPtr, Ordering}};
use core::time::Duration;
use core::pin::Pin;
use core::num::NonZero;
use core::sync::atomic::*;
use core::sync::atomic::Ordering::Acquire;
use core::{ffi::c_void, sync::atomic::AtomicU32};

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let channel = Channel::new_raw();
    (Sender::new(channel), Receiver::new(channel))
}

use alloc::{boxed::Box, vec::Vec};
use ntapi::winapi_local::um::winnt::NtCurrentTeb;
use toolkit::{Arc, println};
use winapi::um::synchapi::{WaitOnAddress, WakeByAddressSingle};

use crate::{backoff::Backoff, futex::{wait_on_address, wake_by_address_single}, instant::Instant, mutex::Mutex};

const LAP: usize = 32;
const BLOCK_CAP: usize = LAP - 1;

#[repr(align(64))]
pub struct PositionPadded<T> {
    index: Atomic<usize>,
    block: Atomic<*mut Block<T>>,
}

impl<T> PositionPadded<T> {
    pub fn new() -> Self {
        Self {
            block: core::sync::atomic::AtomicPtr::new(core::ptr::null_mut()),
            index: AtomicUsize::new(0),
        }
    }
}

pub struct Channel<T> {
    senders: Atomic<usize>,
    channel_receivers: Atomic<usize>,
    destroy: Atomic<bool>,
    head: PositionPadded<T>,
    tail: PositionPadded<T>,
    counter_receivers: SyncWaker,
}

impl<T> Channel<T> {
    pub fn new_raw() -> *mut Self {
        Box::into_raw(Box::new(Self {
            senders: AtomicUsize::new(1),
            channel_receivers: AtomicUsize::new(1),
            destroy: AtomicBool::new(false),
            counter_receivers: SyncWaker::new(),
            head: PositionPadded::new(),
            tail: PositionPadded::new()
        }))
    }
}

pub struct Sender<T>(*mut Channel<T>);

unsafe impl<T: Send> Send for Sender<T> {}
unsafe impl<T: Send> Sync for Sender<T> {}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        let channel = unsafe {&* self.0 };

        let count = channel.senders.fetch_add(1, Ordering::Relaxed);
        
        if count > isize::MAX as usize {
            core::intrinsics::abort();
        }

        Self(self.0)
    }
}

#[derive(Debug, Default)]
pub(crate) struct ListToken {
    block: *const u8,
    offset: usize,
}

const MARK_BIT: usize = 1;
const SHIFT: usize = 1;
const WRITE: usize = 1;
const READ: usize = 2;
const DESTROY: usize = 4;

impl<T> Sender<T> {
    pub fn new(value: *mut Channel<T>) -> Self {
        Self(value)
    }

    pub fn send(&self, message: T) -> Result<(), SendError> {
        let token = &mut ListToken::default();
        self.start_send(token);
        self.write(token, message);
        Ok(())
    }

    fn start_send(&self, token: &mut ListToken) -> bool {
        let backoff = Backoff::new();
        let channel = unsafe {&* self.0 };
        let mut tail = channel.tail.index.load(Ordering::Acquire);
        let mut block = channel.tail.block.load(Ordering::Acquire);
        let mut next_block = None;

        loop {
            if tail & MARK_BIT != 0 {
                token.block = core::ptr::null();
                return true;
            }

            let offset = (tail >> SHIFT) % LAP;

            if offset == BLOCK_CAP {
                backoff.spin_heavy();
                tail = channel.tail.index.load(Ordering::Acquire);
                block = channel.tail.block.load(Ordering::Acquire);
                continue;
            }

            if offset + 1 == BLOCK_CAP && next_block.is_none() {
                next_block = Some(Block::<T>::new());
            }

            if block.is_null() {
                let new = Box::into_raw(Block::<T>::new());

                if channel
                    .tail
                    .block
                    .compare_exchange(block, new, Ordering::Release, Ordering::Relaxed)
                    .is_ok()
                {
                    channel.head.block.store(new, Ordering::Release);
                    block = new;
                } else {
                    next_block = unsafe { Some(Box::from_raw(new)) };
                    tail = channel.tail.index.load(Ordering::Acquire);
                    block = channel.tail.block.load(Ordering::Acquire);
                    continue;
                }
            }

            let new_tail = tail + (1 << SHIFT);

            match channel.tail.index.compare_exchange_weak(
                tail,
                new_tail,
                Ordering::SeqCst,
                Ordering::Acquire,
            ) {
                Ok(_) => unsafe {
                    if offset + 1 == BLOCK_CAP {
                        let next_block = Box::into_raw(next_block.unwrap());
                        channel.tail.block.store(next_block, Ordering::Release);
                        channel.tail.index.fetch_add(1 << SHIFT, Ordering::Release);
                        (*block).next.store(next_block, Ordering::Release);
                    }

                    token.block = block as *const u8;
                    token.offset = offset;
                    return true;
                },
                Err(_) => {
                    backoff.spin_light();
                    tail = channel.tail.index.load(Ordering::Acquire);
                    block = channel.tail.block.load(Ordering::Acquire);
                }
            }
        }
    }

    pub fn write(&self, token: &mut ListToken, msg: T) -> Result<(), T> {

        if token.block.is_null() {
            return Err(msg);
        }

        let block = token.block as *mut Block<T>;
        let offset = token.offset;
        let channel = unsafe { &mut *self.0 };

        unsafe {
            let slot = (*block).slots.get_unchecked(offset);
            slot.msg.get().write(MaybeUninit::new(msg));
            slot.state.fetch_or(WRITE, Ordering::Release);
        }

        channel.counter_receivers.notify();

        Ok(())
    }
}

#[derive(Debug)]
pub enum SendError {

}

pub struct Receiver<T>(*mut Channel<T>);

#[derive(Debug)]
pub enum RecvError {
    Disconnected
}

impl<T> Receiver<T> {
    pub fn new(value: *mut Channel<T>) -> Self {
        Self(value)
    }

    pub fn recv(&self) -> Result<T, RecvError> {
        let token = &mut ListToken::default();
        let channel = unsafe { &mut *self.0 };

        loop {
            if self.start_recv(token) {
                unsafe {
                    return self.read(token).map_err(|_| RecvError::Disconnected);
                }
            }

            let cx = Context::get_from_teb_or_create();
            let oper = Operation::hook(token);
            channel.counter_receivers.register(oper, cx);

            if !self.is_empty() || self.is_disconnected() {
                let _ = cx.try_select(Selected::Aborted);
            }

            let sel = unsafe { cx.wait_until(None) };

            match sel {
                Selected::Waiting => unreachable!(),
                Selected::Aborted | Selected::Disconnected => {
                    channel.counter_receivers.unregister(oper).unwrap();
                }
                Selected::Operation(_) => {}
            }
        }
    }

    pub(crate) fn is_disconnected(&self) -> bool {
        let channel = unsafe { &mut *self.0 };
        channel.tail.index.load(Ordering::SeqCst) & MARK_BIT != 0
    }

    pub fn is_empty(&self) -> bool {
        let channel = unsafe { &mut *self.0 };
        let head = channel.head.index.load(Ordering::SeqCst);
        let tail = channel.tail.index.load(Ordering::SeqCst);
        head >> SHIFT == tail >> SHIFT
    }

    fn start_recv(&self, token: &mut ListToken) -> bool {
        let backoff = Backoff::new();
        let channel = unsafe { &mut *self.0 };
        
        let mut head = channel.head.index.load(Ordering::Acquire);
        let mut block = channel.head.block.load(Ordering::Acquire);

        loop {
            let offset = (head >> SHIFT) % LAP;

            if offset == BLOCK_CAP {
                backoff.spin_heavy();
                head = channel.head.index.load(Ordering::Acquire);
                block = channel.head.block.load(Ordering::Acquire);
                continue;
            }

            let mut new_head = head + (1 << SHIFT);

            if new_head & MARK_BIT == 0 {
                core::sync::atomic::fence(Ordering::SeqCst);
                let tail = channel.tail.index.load(Ordering::Relaxed);

                if head >> SHIFT == tail >> SHIFT {
                    if tail & MARK_BIT != 0 {
                        token.block = core::ptr::null();
                        return true;
                    } else {
                        return false;
                    }
                }

                if (head >> SHIFT) / LAP != (tail >> SHIFT) / LAP {
                    new_head |= MARK_BIT;
                }
            }

            if block.is_null() {
                backoff.spin_heavy();
                head = channel.head.index.load(Ordering::Acquire);
                block = channel.head.block.load(Ordering::Acquire);
                continue;
            }

            match channel.head.index.compare_exchange_weak(
                head,
                new_head,
                Ordering::SeqCst,
                Ordering::Acquire,
            ) {
                Ok(_) => unsafe {
                    if offset + 1 == BLOCK_CAP {
                        let next = (*block).wait_next();
                        let mut next_index = (new_head & !MARK_BIT).wrapping_add(1 << SHIFT);
                        if !(*next).next.load(Ordering::Relaxed).is_null() {
                            next_index |= MARK_BIT;
                        }

                        channel.head.block.store(next, Ordering::Release);
                        channel.head.index.store(next_index, Ordering::Release);
                    }

                    token.block = block as *const u8;
                    token.offset = offset;
                    return true;
                },
                Err(_) => {
                    backoff.spin_light();
                    head = channel.head.index.load(Ordering::Acquire);
                    block = channel.head.block.load(Ordering::Acquire);
                }
            }
        }
    }

    pub(crate) unsafe fn read(&self, token: &mut ListToken) -> Result<T, ()> {
        if token.block.is_null() {
            return Err(());
        }

        let block = token.block as *mut Block<T>;
        let offset = token.offset;
        unsafe {
            let slot = (*block).slots.get_unchecked(offset);
            slot.wait_write();
            let msg = slot.msg.get().read().assume_init();

            if offset + 1 == BLOCK_CAP {
                Block::destroy(block, 0);
            } else if slot.state.fetch_or(READ, Ordering::AcqRel) & DESTROY != 0 {
                Block::destroy(block, offset + 1);
            }

            Ok(msg)
        }
    }
}

pub struct SyncWaker {
    inner: Mutex<Waker>,
    is_empty: Atomic<bool>,
}

impl SyncWaker {
    pub fn new() -> Self {
        SyncWaker { inner: Mutex::new(Waker::new()), is_empty: AtomicBool::new(true) }
    }

    pub fn register(&self, oper: Operation, cx: &Context) {
        let mut inner = self.inner.lock();
        inner.register(oper, cx);
        self.is_empty
            .store(inner.selectors.is_empty() && inner.observers.is_empty(), Ordering::SeqCst);
    }

    pub fn unregister(&self, oper: Operation) -> Option<Entry> {
        let mut inner = self.inner.lock();
        let entry = inner.unregister(oper);
        self.is_empty
            .store(inner.selectors.is_empty() && inner.observers.is_empty(), Ordering::SeqCst);
        entry
    }

    pub fn notify(&mut self) {
        if !self.is_empty.load(Ordering::SeqCst) {
            let mut inner = self.inner.lock();
            if !self.is_empty.load(Ordering::SeqCst) {
                inner.try_select();
                inner.notify();
                self.is_empty.store(
                    inner.selectors.is_empty() && inner.observers.is_empty(),
                    Ordering::SeqCst,
                );
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Operation(usize);

impl Operation {
    #[inline]
    pub fn hook<T>(r: &mut T) -> Operation {
        let val = (r as *mut T).addr();
        assert!(val > 2);
        Operation(val)
    }
}

pub struct Entry {
    pub oper: Operation,
    pub packet: *mut (),
    pub cx: Context,
}

pub struct Waker {
    selectors: Vec<Entry>,
    observers: Vec<Entry>,
}

impl Waker {
    pub(crate) fn new() -> Self {
        Waker { selectors: Vec::new(), observers: Vec::new() }
    }

    pub fn register(&mut self, oper: Operation, cx: &Context) {
        self.register_with_packet(oper, core::ptr::null_mut(), cx);
    }

    pub(crate) fn register_with_packet(&mut self, oper: Operation, packet: *mut (), cx: &Context) {
        self.selectors.push(Entry { oper, packet, cx: cx.clone() });
    }

    pub(crate) fn unregister(&mut self, oper: Operation) -> Option<Entry> {
        if let Some((i, _)) =
            self.selectors.iter().enumerate().find(|&(_, entry)| entry.oper == oper)
        {
            let entry = self.selectors.remove(i);
            Some(entry)
        } else {
            None
        }
    }

    pub fn try_select(&mut self) -> Option<Entry> {
        if self.selectors.is_empty() {
            None
        } else {
            let thread_id = Context::current_thread_id();

            self.selectors
                .iter()
                .position(|selector| {
                    selector.cx.thread_id() != thread_id
                        && selector // Try selecting this operation.
                            .cx
                            .try_select(Selected::Operation(selector.oper))
                            .is_ok()
                        && {
                            selector.cx.store_packet(selector.packet);
                            selector.cx.unpark();
                            true
                        }
                })

                .map(|pos| self.selectors.remove(pos))
        }
    }

    pub fn notify(&mut self) {
        for entry in self.observers.drain(..) {
            if entry.cx.try_select(Selected::Operation(entry.oper)).is_ok() {
                entry.cx.unpark();
            }
        }
    }

    pub fn disconnect(&mut self) {
        for entry in self.selectors.iter() {
            if entry.cx.try_select(Selected::Disconnected).is_ok() {
                entry.cx.unpark();
            }
        }

        self.notify();
    }
}

impl Drop for Waker {
    fn drop(&mut self) {
        debug_assert_eq!(self.selectors.len(), 0);
        debug_assert_eq!(self.observers.len(), 0);
    }
}

struct Slot<T> {
    msg: UnsafeCell<MaybeUninit<T>>,
    state: Atomic<usize>,
}

impl<T> Slot<T> {
    fn wait_write(&self) {
        let backoff = Backoff::new();
        while self.state.load(Ordering::Acquire) & WRITE == 0 {
            backoff.spin_heavy();
        }
    }
}

struct Block<T> {
    next: Atomic<*mut Block<T>>,
    slots: [Slot<T>; BLOCK_CAP],
}

impl<T> Block<T> {
    fn new() -> Box<Block<T>> {
        unsafe { Box::new_zeroed().assume_init() }
    }

    fn wait_next(&self) -> *mut Block<T> {
        let backoff = Backoff::new();
        loop {
            let next = self.next.load(Ordering::Acquire);
            if !next.is_null() {
                return next;
            }
            backoff.spin_heavy();
        }
    }

    unsafe fn destroy(this: *mut Block<T>, start: usize) {
        for i in start..BLOCK_CAP - 1 {
            let slot = unsafe { (*this).slots.get_unchecked(i) };

            if slot.state.load(Ordering::Acquire) & READ == 0
                && slot.state.fetch_or(DESTROY, Ordering::AcqRel) & READ == 0
            {
                return;
            }
        }

        drop(unsafe { Box::from_raw(this) });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selected {
    Waiting,
    Aborted,
    Disconnected,
    Operation(Operation),
}

impl From<usize> for Selected {
    #[inline]
    fn from(val: usize) -> Selected {
        match val {
            0 => Selected::Waiting,
            1 => Selected::Aborted,
            2 => Selected::Disconnected,
            oper => Selected::Operation(Operation(oper)),
        }
    }
}

impl Into<usize> for Selected {
    #[inline]
    fn into(self) -> usize {
        match self {
            Selected::Waiting => 0,
            Selected::Aborted => 1,
            Selected::Disconnected => 2,
            Selected::Operation(Operation(val)) => val,
        }
    }
}

#[derive(Clone)]
pub struct Context(Arc<Inner>);

struct Inner {
    select: Atomic<usize>,
    packet: Atomic<*mut ()>,
    thread: Thread,
    thread_id: usize,
}

const PARKED: u8 = u8::MAX;
const EMPTY: u8 = 0;
const NOTIFIED: u8 = 1;

pub struct Thread(Pin<Arc<ThreadInner>>);

#[repr(align(8))]
pub struct ThreadInner {
    name: Option<Box<[u8]>>,
    id: NonZero<u64>,
    parker: Parker,
}

#[repr(align(8))]
pub struct Parker(AtomicU8);

impl Parker {
    pub const fn new() -> Self {
        Self(AtomicU8::new(EMPTY))
    }

    pub unsafe fn park(self: Pin<&Self>) {
        
        if self.0.fetch_sub(1, Acquire) == NOTIFIED {
            return;
        }

        loop {

            wait_on_address(&self.0, PARKED, None);
            
            if self.0.compare_exchange(NOTIFIED, EMPTY, Acquire, Acquire).is_ok() {
                return;
            }
        }
    }

    pub fn unpark(self: Pin<&Self>) {
        if self.0.swap(NOTIFIED, Ordering::Release) == PARKED {
            wake_by_address_single(&self.0);
        }
    }

    pub fn park_timeout(self: Pin<&Self>, timeout: Duration) {
        
        if self.0.fetch_sub(1, Acquire) == NOTIFIED {
            return;
        }

        wait_on_address(&self.0, PARKED, Some(timeout));
        
        if self.0.swap(EMPTY, Acquire) == NOTIFIED {
        }
    }
}

impl Thread {
    pub fn new() -> Self {
        static COUNTER: Atomic<u64> = AtomicU64::new(0);

        let mut last = COUNTER.load(Ordering::Relaxed);
        let mut new_id = 0;

        loop {
            new_id = last.checked_add(1).unwrap();

            match COUNTER.compare_exchange_weak(last, new_id, Ordering::Relaxed, Ordering::Relaxed) {
                Ok(_) => break,
                Err(id) => last = id,
            }
        }
        println!("{}", new_id);
        let inner = unsafe {
            let mut arc = Arc::<ThreadInner>::new_uninit_in();
            let ptr = Arc::get_mut_unchecked(&mut arc).as_mut_ptr();
            (&raw mut (*ptr).name).write(None);
            (&raw mut (*ptr).id).write(NonZero::new(new_id).unwrap());
            (&raw mut (*ptr).parker).write(Parker::new());
            Pin::new_unchecked(arc.assume_init())
        };

        Self(inner)
    }

    pub fn park(&self) {
        let inner = self.0.as_ref();
        let parker = unsafe { Pin::map_unchecked(inner, |inner| &inner.parker) };
        unsafe { parker.park() };
    }

    pub fn unpark(&self) {
        let inner = self.0.as_ref();
        let parker = unsafe { Pin::map_unchecked(inner, |inner| &inner.parker) };
        parker.unpark();
    }

    pub fn park_timeout(&self, dur: Duration) {
        let inner = self.0.as_ref();
        let parker = unsafe { Pin::map_unchecked(inner, |inner| &inner.parker) };
        parker.park_timeout(dur);
    }
}

impl Context {

    pub fn get_from_teb_or_create() -> &'static Self {
        let teb = unsafe { NtCurrentTeb() };
        let slot_ptr = unsafe { &(*teb).TlsSlots[0] };
        
        if slot_ptr.is_null() {
            let thread = Thread::new();
            let inner = Inner {
                select: AtomicUsize::new(Selected::Waiting.into()),
                packet: AtomicPtr::new(core::ptr::null_mut()),
                thread,
                thread_id: Context::current_thread_id(),
            };
            
            let context = Context(Arc::new(inner));
            let static_context: &'static mut Context = Box::leak(Box::new(context));
            
            unsafe {
                (*teb).TlsSlots[0] = static_context as *mut Context as *mut () as _;
            }
            
            static_context
        } else {
            let context_ptr = *slot_ptr as *mut Context;
            unsafe { &mut *context_ptr }
        }
    }

    pub fn thread_id(&self) -> usize {
        let teb = unsafe { NtCurrentTeb() };
        unsafe { (*teb).ClientId.UniqueThread as _ }
    }

    pub fn current_thread_id() -> usize {
        let teb = unsafe { NtCurrentTeb() };
        unsafe { (*teb).ClientId.UniqueThread as _ }
    }

    pub fn unpark(&self) {
        self.0.thread.unpark();
    }

    pub fn store_packet(&self, packet: *mut ()) {
        if !packet.is_null() {
            self.0.packet.store(packet, Ordering::Release);
        }
    }

    #[inline]
    pub fn try_select(&self, select: Selected) -> Result<(), Selected> {
        self.0.select
            .compare_exchange(
                Selected::Waiting.into(),
                select.into(),
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map(|_| ())
            .map_err(|e| e.into())
    }

    pub unsafe fn wait_until(&self, deadline: Option<Instant>) -> Selected {
        loop {
            let sel = Selected::from(self.0.select.load(Ordering::Acquire));
            if sel != Selected::Waiting {
                return sel;
            }

            if let Some(end) = deadline {
                let now = Instant::now();

                if now < end {
                    self.0.thread.park_timeout(end - now);
                } else {
                    return match self.try_select(Selected::Aborted) {
                        Ok(()) => Selected::Aborted,
                        Err(s) => s,
                    };
                }
            } else {
                self.0.thread.park();
            }
        }
    }
}
