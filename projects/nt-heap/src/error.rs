use winapi::shared::ntdef::NTSTATUS;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeapError(pub NTSTATUS);

impl HeapError {
    pub fn status(&self) -> NTSTATUS {
        self.0
    }
}

pub type HeapResult<T> = Result<T, HeapError>;