# Freelist Text Allocator

A minimal `no_std` memory allocator for Windows, using a lock‑free free‑list stored in a custom `.text` arena.

## Overview

This crate provides a custom global allocator that operates on a fixed‑size arena embedded directly in the executable’s `.text` section. It is designed for low‑level systems programming where the standard library is unavailable and full control over memory layout is required.

The allocator maintains a singly‑linked list of free blocks using atomic pointers, making it thread‑safe without any locks. The arena memory is marked executable/writable via `NtProtectVirtualMemory`, allowing the data to be stored in code sections (a common trick for shellcode‑like scenarios).
