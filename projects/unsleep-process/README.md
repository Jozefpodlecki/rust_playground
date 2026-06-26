# 📦 Unsleep Process

A minimal `no_std` Windows executable demonstrating low-level thread synchronization using NT API functions. This project explores how to wake a thread from an alertable wait state using `NtAlertResumeThread`.

## 🎯 Overview

The main thread enters an alertable wait state using `NtDelayExecution(1, &delay)` with `Alertable = TRUE`, which would normally wait forever (or until the system shuts down). A spawned worker thread then wakes it up after a short delay.

The challenge was finding the right NT API function to wake the main thread:

| Function | Result |
|----------|--------|
| `NtAlertThread` | ❌ Returns `STATUS_ACCESS_DENIED` (-1073741790) even with proper handle |
| `NtAlertResumeThread` | ✅ Successfully wakes the main thread |