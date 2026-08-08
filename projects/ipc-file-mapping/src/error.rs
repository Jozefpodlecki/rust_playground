use core::fmt;

use winapi::{shared::ntstatus::STATUS_UNSUCCESSFUL, um::winnt::STATUS_INVALID_HANDLE};


pub enum MainError {
    Server(ServerError),
    Client(ClientError)
}

impl fmt::Display for MainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MainError::Server(err) => write!(f, "Server error: {}", err),
            MainError::Client(err) => write!(f, "Client error: {}", err),
        }
    }
}

impl MainError {
    pub fn to_code(&self) -> u32 {
        STATUS_UNSUCCESSFUL as _
    }
}

impl From<ServerError> for MainError {
    fn from(err: ServerError) -> Self {
        MainError::Server(err)
    }
}

impl From<ClientError> for MainError {
    fn from(err: ClientError) -> Self {
        MainError::Client(err)
    }
}

#[derive(Debug)]
pub enum ServerError {
    CreateFileMappingFailed(u32),
    MapViewOfFileFailed(u32),
    CreateMutexFailed,
    CreateEventFailed,
    CreateJobObjectFailed,
    SetJobInformationFailed,
    AssignProcessToJobObjectFailed,
    CreateProcessFailed(u32),
    LockFailed,
    SharedMemory(SharedMemoryError)
}

impl From<SharedMemoryError> for ServerError {
    fn from(err: SharedMemoryError) -> Self {
        ServerError::SharedMemory(err)
    }
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::CreateFileMappingFailed(code) => write!(f, "Failed to create file mapping: {code}"),
            ServerError::MapViewOfFileFailed(code) => write!(f, "Failed to map view of file {code}"),
            ServerError::CreateMutexFailed => write!(f, "Failed to create mutex"),
            ServerError::CreateEventFailed => write!(f, "Failed to create event"),
            ServerError::CreateJobObjectFailed => write!(f, "Failed to create job object"),
            ServerError::SetJobInformationFailed => write!(f, "Failed to set job information"),
            ServerError::CreateProcessFailed(code) => write!(f, "Failed to create child process: {code}"),
            ServerError::LockFailed => write!(f, "Failed to acquire mutex lock"),
            ServerError::SharedMemory(err) => write!(f, "{err}"),
            ServerError::AssignProcessToJobObjectFailed => write!(f, "Failed to assign process to job object"),
        }
    }
}

impl core::error::Error for ServerError {}


#[derive(Debug)]
pub enum ClientError {
    OpenFileMappingFailed(u32),
    MapViewOfFileFailed,
    OpenMutexFailed,
    OpenEventFailed,
    WaitFailed,
    LockFailed,
    SharedMemory(SharedMemoryError)
}

impl From<SharedMemoryError> for ClientError {
    fn from(e: SharedMemoryError) -> Self {
        ClientError::SharedMemory(e)
    }
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::OpenFileMappingFailed(code) => write!(f, "Failed to open file mapping: {code}"),
            ClientError::MapViewOfFileFailed => write!(f, "Failed to map view of file"),
            ClientError::OpenMutexFailed => write!(f, "Failed to open mutex"),
            ClientError::OpenEventFailed => write!(f, "Failed to open event"),
            ClientError::WaitFailed => write!(f, "Failed to wait for event"),
            ClientError::LockFailed => write!(f, "Failed to acquire mutex lock"),
            ClientError::SharedMemory(err) => write!(f, "{err}"),
        }
    }
}

impl core::error::Error for ClientError {}

#[derive(Debug)]
pub enum SharedMemoryError {
    LockTimeout,
    LockFailed,
    SignalFailed,
    WaitTimeout,
    WaitFailed,
    MessageTooLong,
    InvalidUtf8,
}

impl fmt::Display for SharedMemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SharedMemoryError::LockTimeout => write!(f, "Mutex lock timeout"),
            SharedMemoryError::LockFailed => write!(f, "Mutex lock failed"),
            SharedMemoryError::WaitTimeout => write!(f, "Event wait timeout"),
            SharedMemoryError::WaitFailed => write!(f, "Event wait failed"),
            SharedMemoryError::MessageTooLong => write!(f, "Message exceeds maximum size"),
            SharedMemoryError::InvalidUtf8 => write!(f, "Message contains invalid UTF-8"),
            SharedMemoryError::SignalFailed => write!(f, "Event signal failed"),
        }
    }
}

impl core::error::Error for SharedMemoryError {}