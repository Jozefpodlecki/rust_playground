use alloc::boxed::Box;
use core::cell::SyncUnsafeCell;

use heapless::{LinearMap, linear_map::Entry};
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

    pub fn reset(&self, tid: u32) {
        let mut guard = self.0.write();
        match guard.entry(tid) {
            Entry::Occupied(mut entry) => {
                let thread = entry.get_mut();
                thread.timestamp();
                thread.verdict = DebugVerdict { tid, kind: DebugVerdictKind::None };
            }
            Entry::Vacant(entry) => {
                entry.insert(Box::new(DebugThread::new(tid)));
            }
        }
    }

    pub fn ptr(&self, tid: u32) -> Option<*const DebugThread> {
        let guard = self.0.read();
        guard.get(&tid).map(|b| b.as_ptr() as _)
    }

    pub fn verdict(&self, tid: u32) -> Option<DebugVerdict> {
        self.0.read().get(&tid).map(|b| b.verdict)
    }

    pub fn set_verdict(&self, verdict: DebugVerdict) -> Option<*const DebugThread> {
        let mut guard = self.0.write();
        let entry = guard.get_mut(&verdict.tid)?;
        entry.updated_on = KUserSharedData::system_time();
        entry.verdict = verdict;
        Some(entry.as_ptr() as _)
    }
}

pub struct DebugThread {
    pub updated_on: SystemTime,
    pub verdict: DebugVerdict,
}

impl DebugThread {
    pub const fn new(tid: u32) -> Self {
        Self {
            updated_on: KUserSharedData::system_time(),
            verdict: DebugVerdict { tid, kind: DebugVerdictKind::None },
        }
    }

    pub fn as_ptr(&self) -> PVOID {
        &self as *const _ as _
    }

    pub fn timestamp(&mut self) {
        self.updated_on = KUserSharedData::system_time();
    }
}