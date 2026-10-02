use alloc::boxed::Box;
use core::cell::SyncUnsafeCell;

use heapless::LinearMap;
use win_platform::{KUserSharedData, types::{PVOID, SystemTime}};

use crate::{api::AsyncIpcServer, types::*};

pub const MAX_THREADS: usize = 40;

pub struct ServerCell(SyncUnsafeCell<Option<AsyncIpcServer>>);

impl ServerCell {
    pub const fn new() -> Self {
        Self(SyncUnsafeCell::new(None))
    }

    pub fn set(&self, server: AsyncIpcServer) {
        unsafe { *self.0.get() = Some(server) }
    }

    pub fn get(&self) -> &AsyncIpcServer {
        unsafe { (*self.0.get()).as_ref().unwrap_unchecked() }
    }
}

pub static SERVER: ServerCell = ServerCell::new();
pub static REGISTRY: ThreadRegistry = ThreadRegistry::new();

pub struct ThreadRegistry(pub spin::RwLock<LinearMap<u32, Box<DebugThread>, MAX_THREADS>>);

impl ThreadRegistry {
    pub const fn new() -> Self {
        Self(spin::RwLock::new(LinearMap::new()))
    }
}

pub struct DebugThread {
    pub updated_on: SystemTime,
    pub verdict: DebugVerdict,
}

impl DebugThread {
    pub fn as_ptr(&self) -> PVOID {
        &self as *const _ as _
    }

    pub fn timestamp(&mut self) {
        self.updated_on = KUserSharedData::system_time();
    }
}