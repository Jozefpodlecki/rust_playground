# Custom Loggers

A collection of custom Rust loggers implementing the [`log`](https://crates.io/crates/log) facade.

This project currently provides two logging backends:

1. **NT File Logger**  
   Writes application logs directly to a file using native Windows file handling.

2. **Windows Event Viewer Logger**  
   Publishes events through Windows Event Tracing (ETW), making logs available in Windows Event Viewer.

---

## Features

- Implements the standard Rust `log` interface.
- Supports log levels:
  - Error
  - Warning
  - Info
  - Debug
  - Trace
- Uses native Windows APIs.
- Supports:
  - File based logging
  - Windows Event Viewer logging

---

# NT File Logger

The file logger creates a log file next to the running executable.

## Build Integration

The project automatically generates and embeds the ETW provider resources during the Rust build process.

The `build.rs` script performs the following steps:

1. Runs the Windows Message Compiler (`mc.exe`) against the provider manifest.
2. Generates:
   - Resource script (`provider.rc`)
   - Binary message resources
   - Provider metadata resources
3. Places generated files into the project build output directory.
4. Compiles and embeds the generated resources into the final executable using `embed_resource`.