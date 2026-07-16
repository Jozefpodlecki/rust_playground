# 📦 Stack Alloc

A `#![no_std]` custom memory allocator that repurposes a stack-allocated buffer as a heap.

## The Idea

Instead of relying on the OS for dynamic memory, we carve a static chunk of memory (the **arena**) and manage it ourselves using a **free-list allocator**. The allocator hands out blocks from this arena, tracks free space, and merges adjacent free blocks to prevent fragmentation.

## How It Works

1. **Arena**: A fixed-size byte array allocated on the stack.
2. **Free List**: A linked list of free memory blocks, each storing a pointer to the next free block and its size.
3. **Allocation**: Scans the free list for a block large enough → splits it if needed.
4. **Deallocation**: Returns the block to the free list and **coalesces** (merges) adjacent free blocks.
5. **Reallocation**: Tries to allocate a new block; if that fails, it frees the old block, coalesces, and retries. Uses a temporary stack buffer to preserve data when moving blocks.

## Usage

Simply call `init_arena!(SIZE)` at the start of your entrypoint to create the arena and hook up the allocator:

```rust
init_arena!(4096);
```