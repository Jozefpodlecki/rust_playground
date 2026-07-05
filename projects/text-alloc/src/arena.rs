use core::{arch::naked_asm, ptr::null_mut, sync::atomic::{AtomicPtr, Ordering}};

use crate::types::ArenaPtr;

pub const ARENA_SIZE: usize = 10 * 16;

#[unsafe(naked)]
pub unsafe extern "system" fn arena_memory() {
    naked_asm!(
        ".octa 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        ".octa 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        ".octa 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        ".octa 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        ".octa 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        ".octa 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        ".octa 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        ".octa 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        ".octa 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        ".octa 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        // "addq $0x7FFFFFFF, %gs:0x7FFFFFFF(%rsp)",
    );
}

impl ArenaPtr {
    pub fn get() -> Self {
        Self(arena_memory as *const () as *mut u8)
    }

    pub const fn size(&self) -> usize {
        ARENA_SIZE
    }
}