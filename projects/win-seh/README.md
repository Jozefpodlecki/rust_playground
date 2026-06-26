# 📦 Windows Structured Exception Handler

A lightweight, `no_std` implementation of Windows Structured Exception Handling (SEH) built from scratch in Rust. This project demonstrates dynamic registration of exception handlers, runtime code generation, and low-level Windows internals.

## Overview

This crate implements a custom exception handler that can catch and handle various CPU exceptions (access violations, divide by zero, breakpoints, privileged instructions, etc.) by dynamically registering function tables with the Windows kernel.

### What It Does

- Allocates executable memory using `NtAllocateVirtualMemory`
- Writes dynamic machine code (shellcode) into the allocated buffer
- Builds `RUNTIME_FUNCTION` and `UNWIND_INFO` structures manually
- Registers the function table with `RtlAddFunctionTable`
- Catches exceptions in a custom handler
- Modifies the context (`Rip`) to skip faulting instructions
- Resumes execution seamlessly

## References

- [Exception Handlers - Uninformed](http://uninformed.org/index.cgi?v=4&a=1&p=18)
- [Windows RtlAddFunctionTable pmeerw's blog](https://pmeerw.net/blog/programming/RtlAddFunctionTable.html)