# 🧵 Unsleep Process

A minimal `no_std` Windows executable demonstrating low-level thread manipulation using the NT API. Explores thread synchronization, context migration, and stack copying.

## 🎯 Overview

The project demonstrates two approaches to handling threads stuck in infinite waits:

| Approach | Description | Method |
|----------|-------------|--------|
| **Alertable Wake** | Thread in alertable wait (`NtDelayExecution(1, INFINITE)`) | `NtAlertResumeThread` wakes it cleanly ✅ |
| **Thread Hijack** | Thread in non-alertable wait (`NtDelayExecution(0, INFINITE)`) | Copy stack/context to new thread, terminate original |