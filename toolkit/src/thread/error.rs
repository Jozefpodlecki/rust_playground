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