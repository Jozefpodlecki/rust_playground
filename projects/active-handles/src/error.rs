use thiserror::Error;

#[derive(Error, Debug)]
pub enum NtApiError {
    #[error("Buffer too small: needed {needed} bytes, available {available} bytes")]
    BufferTooSmall { needed: usize, available: usize },
    
    #[error("NtQuerySystemInformation failed with status: {0:X}")]
    QueryFailed(i32),
}