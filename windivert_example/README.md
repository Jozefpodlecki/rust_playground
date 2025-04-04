# 🌐 Network Packet Capture Application

This is an example application that captures and processes network packets using the `WinDivert` library.

It is designed to run on Windows and listens for network packets on a specified port.

The application can be easily extended for packet manipulation, analysis, or any other related network tasks.

## 🚀 Features
- 🛠️ **Packet Capture**: Listens for network packets on a specified port (default: port 443).
- ⚡ **Async Processing**: Utilizes Rust's `tokio` runtime for asynchronous, non-blocking operations.
- 🔄 **Graceful Shutdown**: Stops packet capture and processing after a set duration (default: 3 seconds).
- 📝 **Logging**: Logs key events and errors for better visibility.

## 🔑 How It Works
1. **Initialization**: The program sets up logging and initializes the consumer for packet processing.
2. **Packet Capture**: Captures network packets on the specified port using `WinDivert`.
3. **Processing**: Processes each captured packet asynchronously in a loop.
4. **Graceful Shutdown**: After a predefined duration, the application stops packet capture and gracefully exits.

## ⚙️ Prerequisites
- **Windows OS**: This application uses the `WinDivert` library, which is designed to work on Windows.
- **Rust**: You need to have Rust installed to build and run the application.
- **Elevated Permissions**: The app requires elevated (administrator) permissions to access and capture network packets on Windows.

## 🏁 Getting Started

### 1. Clone the Repository
Clone this project to your local machine:

```bash
git clone https://github.com/Jozefpodlecki/rust_playground.git
cd rust_playground/windivert_example
```

### 2. Build the Project & Run the Application

```bash
cargo build
cd target/debug
.\windivert_example
```