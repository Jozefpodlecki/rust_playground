
# Single Thread Runtime

A minimal `no_std` async runtime for Windows.

## Overview

A lightweight, single-threaded async runtime. Provides core async execution without requiring the standard library.

## Features

- No_std compatible
- Single-threaded executor
- Task spawning with spawn()
- JoinHandle for getting results from spawned tasks
- Join combinator for waiting on multiple futures
- Timer support with Delay
- Custom waker with task re-queuing
- High-resolution timing via Windows KUSER_SHARED_DATA
- Minimal dependencies (ntapi, emballoc)

## Usage