---
id: background_worker
title: "Background Worker"
difficulty: "easy"
tags: ["threading", "concurrency", "basic"]
---

# Background Worker

Create a simple Rust program that spawns a background thread to perform some work while the main thread continues executing.

## ✅ Requirements

Write a Rust program that:

- Starts a background thread that prints `"Working..."` 5 times with a 1-second delay.
- While the background thread is working, the main thread should print `"Main thread is free!"` every second.
- After the background thread finishes, the main thread should wait for it using `.join()`.

## 📦 Example Output

```bash
Main thread is free!
Working...
Main thread is free!
Working...
...
Background thread finished!
```