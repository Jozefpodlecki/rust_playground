#![no_std]
#![no_main]
#![windows_subsystem = "console"]
#![allow(internal_features)]
#![feature(unsafe_cell_access)]
#![feature(generic_atomic)]
#![feature(core_intrinsics)]

use core::fmt;

use toolkit::{Sleeper, println};

use crate::custalloc::CustAllocator;

#[macro_use]
extern crate alloc;

#[inline(never)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

extern crate builtins;

#[global_allocator]
static ALLOCATOR: CustAllocator<8192> = CustAllocator::new();

mod channel;
mod mutex;
mod backoff;
mod instant;
mod custalloc;
mod futex;

#[derive(Clone, Copy, Debug)]
pub struct RingArray<const N: usize> {
    data: [u8; N],
    pos: usize,
}

impl<const N: usize> RingArray<N> {
    pub const fn new() -> Self {
        Self {
            data: [0; N],
            pos: 0,
        }
    }

    pub fn next(&mut self) -> [u8; N] {
        self.data = [0; N];
        self.data[self.pos] = 1;
        self.pos = (self.pos + 1) % N;
        self.data
    }

    pub fn current(&self) -> [u8; N] {
        let mut data = [0; N];
        data[self.pos] = 1;
        data
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn reset(&mut self) {
        self.pos = 0;
        self.data = [0; N];
    }
}

impl<const N: usize> Default for RingArray<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Iterator for RingArray<N> {
    type Item = [u8; N];

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next())
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {
    let (tx, rx) = channel::channel::<[u8; 25]>();

    toolkit::Thread::spawn_ex(move || {
        let mut ring = RingArray::<25>::new();
        
        for data in ring {
            tx.send(data).unwrap();
            Sleeper::sleep(100);
        }
    });

    while let Ok(val) = rx.recv() {
        println!("{:?}", val);
    }

    0
}