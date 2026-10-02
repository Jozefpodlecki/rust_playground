use winapi::shared::ntdef::NTSTATUS;

use crate::NtError;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileError {
    Success = 0,
    BufferTooSmall = 0xC0000023,
    ObjectNameNotFound = 0xC0000034,
    ObjectNameInvalid = 0xC0000033,
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
            Self::BufferTooSmall => "Buffer too small",
            Self::ObjectNameNotFound => "Object name not found",
            Self::ObjectNameInvalid => "Object name invalid",
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
            Self::Unspecified => "Unspecified error"
        };
        write!(f, "{}", msg)
    }
}

impl From<NtError> for FileError {
    fn from(err: NtError) -> Self {
        Self::from(err.0)
    }
}

impl From<NtError> for core::io::Error {
    fn from(err: NtError) -> Self {
        nt_status_to_io_err(err.0)
    }
}

const fn nt_status_to_io_err(status: i32) -> core::io::Error {
    use core::io::*;

    match status as u32 {
        0xC000_0005 => const_error!(ErrorKind::PermissionDenied, "STATUS_ACCESS_VIOLATION: invalid handle or lack of write access"),
        0xC000_0022 => const_error!(ErrorKind::PermissionDenied, "STATUS_ACCESS_DENIED: access to the file is denied"),
        0xC000_0008 => const_error!(ErrorKind::InvalidInput,    "STATUS_INVALID_HANDLE: the specified handle is invalid"),
        0xC000_0010 => const_error!(ErrorKind::NotConnected,   "STATUS_DEVICE_NOT_CONNECTED: the device is not connected"),
        0xC000_026E => const_error!(ErrorKind::NotFound,       "STATUS_VOLUME_DISMOUNTED: the volume is not mounted"),
        0xC000_00A2 => const_error!(ErrorKind::WriteZero,      "STATUS_MEDIA_WRITE_PROTECTED: the volume is write-protected"),
        0xC000_0011 => const_error!(ErrorKind::UnexpectedEof,  "STATUS_END_OF_FILE: the end of the file has been reached"),
        0xC000_0034 => const_error!(ErrorKind::NotFound,       "STATUS_OBJECT_NAME_NOT_FOUND"),
        0xC000_003A => const_error!(ErrorKind::NotFound,       "STATUS_OBJECT_PATH_NOT_FOUND"),
        0xC000_0043 => const_error!(ErrorKind::PermissionDenied,"STATUS_SHARING_VIOLATION"),
        0xC000_000D => const_error!(ErrorKind::InvalidInput,   "STATUS_INVALID_PARAMETER"),
        _            => const_error!(ErrorKind::Other,          "NtFlushBuffersFile failed with an unknown status"),
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
            0xC0000033 => Self::ObjectNameInvalid,
            value => {
                Self::Unspecified
            },
        }
    }
}

impl FileError {
    pub fn from_core_io(e: core::io::Error) -> Self {
        match e.kind() {
            core::io::ErrorKind::NotFound              => Self::FileNotFound,
            core::io::ErrorKind::PermissionDenied      => Self::AccessDenied,
            core::io::ErrorKind::ConnectionRefused     => Self::Unspecified,
            core::io::ErrorKind::ConnectionReset       => Self::Unspecified,
            core::io::ErrorKind::HostUnreachable       => Self::Unspecified,
            core::io::ErrorKind::NetworkUnreachable    => Self::Unspecified,
            core::io::ErrorKind::ConnectionAborted     => Self::Unspecified,
            core::io::ErrorKind::NotConnected          => Self::Unspecified,
            core::io::ErrorKind::AddrInUse             => Self::Unspecified,
            core::io::ErrorKind::AddrNotAvailable      => Self::Unspecified,
            core::io::ErrorKind::NetworkDown           => Self::Unspecified,
            core::io::ErrorKind::BrokenPipe            => Self::Unspecified,
            core::io::ErrorKind::AlreadyExists         => Self::NameCollision,
            core::io::ErrorKind::WouldBlock            => Self::Unspecified,
            core::io::ErrorKind::NotADirectory         => Self::PathNotFound,
            core::io::ErrorKind::IsADirectory          => Self::FileIsDirectory,
            core::io::ErrorKind::DirectoryNotEmpty     => Self::Unspecified,
            core::io::ErrorKind::ReadOnlyFilesystem    => Self::AccessDenied,
            // core::io::ErrorKind::FilesystemLoop        => Self::Unspecified,
            core::io::ErrorKind::StaleNetworkFileHandle => Self::Unspecified,
            core::io::ErrorKind::InvalidInput          => Self::InvalidParameter,
            core::io::ErrorKind::InvalidData           => Self::InvalidParameter,
            core::io::ErrorKind::TimedOut              => Self::Unspecified,
            core::io::ErrorKind::WriteZero             => Self::Unspecified,
            core::io::ErrorKind::StorageFull           => Self::Unspecified,
            core::io::ErrorKind::NotSeekable           => Self::Unspecified,
            core::io::ErrorKind::QuotaExceeded         => Self::Unspecified,
            core::io::ErrorKind::FileTooLarge          => Self::Unspecified,
            core::io::ErrorKind::ResourceBusy          => Self::Unspecified,
            core::io::ErrorKind::ExecutableFileBusy    => Self::Unspecified,
            core::io::ErrorKind::Deadlock              => Self::Unspecified,
            core::io::ErrorKind::CrossesDevices        => Self::Unspecified,
            core::io::ErrorKind::TooManyLinks          => Self::Unspecified,
            core::io::ErrorKind::InvalidFilename       => Self::PathSyntaxBad,
            core::io::ErrorKind::ArgumentListTooLong   => Self::InvalidParameter,
            core::io::ErrorKind::Interrupted           => Self::Cancelled,
            core::io::ErrorKind::Unsupported           => Self::Unspecified,
            core::io::ErrorKind::UnexpectedEof         => Self::UnexpectedEof,
            core::io::ErrorKind::OutOfMemory           => Self::Unspecified,
            // core::io::ErrorKind::InProgress            => Self::Unspecified,
            // core::io::ErrorKind::TooManyOpenFiles      => Self::Unspecified,
            // core::io::ErrorKind::InputOutputError      => Self::Unspecified,
            core::io::ErrorKind::Other                 => Self::Unspecified,
            _                                          => Self::Unspecified,
        }
    }
}

impl From<core::io::Error> for FileError {
    fn from(e: core::io::Error) -> Self {
        Self::from_core_io(e)
    }
}