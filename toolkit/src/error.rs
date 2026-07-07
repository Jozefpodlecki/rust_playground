use winapi::shared::ntdef::NTSTATUS;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThreadError {
    /// The thread creation failed with the given NTSTATUS
    CreationFailed(NTSTATUS),
    /// The thread handle was invalid or null
    InvalidHandle,
    /// Failed to wait for the thread to complete
    WaitFailed(NTSTATUS),
    /// The thread exited but did not return a result
    NoResult,
    /// The thread is still running (non-blocking check)
    ThreadRunning,
    /// The thread was terminated before returning a result
    ThreadTerminated,
    /// The join operation failed because the thread is not joinable
    NotJoinable,
    /// The operation timed out
    Timeout,
    /// An invalid argument was provided
    InvalidArgument,
}

impl core::fmt::Display for ThreadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ThreadError::CreationFailed(status) => {
                write!(f, "Thread creation failed with NTSTATUS: 0x{:08X}", status)
            }
            ThreadError::InvalidHandle => {
                write!(f, "Thread handle is invalid or null")
            }
            ThreadError::WaitFailed(status) => {
                write!(f, "Failed to wait for thread with NTSTATUS: 0x{:08X}", status)
            }
            ThreadError::NoResult => {
                write!(f, "Thread exited but did not return a result")
            }
            ThreadError::ThreadRunning => {
                write!(f, "Thread is still running")
            }
            ThreadError::ThreadTerminated => {
                write!(f, "Thread was terminated before returning a result")
            }
            ThreadError::NotJoinable => {
                write!(f, "Thread is not joinable")
            }
            ThreadError::Timeout => {
                write!(f, "Operation timed out")
            }
            ThreadError::InvalidArgument => {
                write!(f, "Invalid argument provided")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileError {
    Success = 0,
    ObjectNameNotFound = 0xC0000034,
    AccessDenied = 0xC0000022,
    FileIsDirectory = 0xC00000CF,
    InvalidParameter = 0xC000000D,
    VolumeDismounted = 0xC000026E,
    FileNotFound = 0xC000000F,
    PathNotFound = 0xC000003A,
    SharingViolation = 0xC0000043,
    BufferOverflow = 0x80000005,
    EndOfFile = 0xC0000011,
    Unspecified = 0xFFFFFFFF,
    PathSyntaxBad = 0xC000003B,
    NameCollision = 0xC0000035,
    UnexpectedEof = 0x99999991,
    Cancelled = 0x99999992,
    InvalidState = 0x99999993,
}

impl core::error::Error for FileError{}

impl core::fmt::Display for FileError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let msg = match self {
            Self::Success => "Success",
            Self::ObjectNameNotFound => "Object name not found",
            Self::AccessDenied => "Access denied",
            Self::FileIsDirectory => "File is a directory",
            Self::InvalidParameter => "Invalid parameter",
            Self::EndOfFile => "End of file",
            Self::FileNotFound => "File not found",
            Self::PathNotFound => "Path not found",
            Self::SharingViolation => "Sharing violation",
            Self::BufferOverflow => "Buffer overflow",
            Self::VolumeDismounted => "Volume Dismounted",
            Self::UnexpectedEof => "Unexpected end of file",
            Self::PathSyntaxBad => "Path syntax bad",
            Self::NameCollision => "Name coliision",
            Self::Cancelled => "Cancelled",
            Self::InvalidState => "Invalid state",
            Self::Unspecified => "Unspecified error",
        };
        write!(f, "{}", msg)
    }
}

impl From<NTSTATUS> for FileError {
    fn from(status: NTSTATUS) -> Self {
        match status as u32 {
            0 => Self::Success,
            0xC0000034 => Self::ObjectNameNotFound,
            0xC0000022 => Self::AccessDenied,
            0xC00000CF => Self::FileIsDirectory,
            0xC000000D => Self::InvalidParameter,
            0xC000026E => Self::VolumeDismounted,
            0xC000000F => Self::FileNotFound,
            0xC000003A => Self::PathNotFound,
            0xC0000043 => Self::SharingViolation,
            0x80000005 => Self::BufferOverflow,
            0xC0000011 => Self::EndOfFile,
            0xC000003B => Self::PathSyntaxBad,
            0xC0000035 => Self::NameCollision,
            value => {
                Self::Unspecified
            },
        }
    }
}