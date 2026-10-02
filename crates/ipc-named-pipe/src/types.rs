use serde::{Deserialize, Serialize};

use crate::error::FrameError;

pub const MAX_FRAME_SIZE: usize = 256;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[derive_const(Default)]
pub struct DebugVerdict {
    pub tid: u32,
    pub kind: DebugVerdictKind
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[derive_const(Default)]
pub enum DebugVerdictKind {
    #[default]
    None,
    Continue,
    TerminateThread { exit_code: i32 },
    TerminateProcess { exit_code: i32 },
    Return
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DebugEvent {
    pub tid: u32,
    pub kind: DebugEventKind
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DebugEventKind {
    Initialized,
    ThreadCreated,
    Breakpoint,
    ProcessExited,
    ThreadExited
}


pub struct Frame {
    data: [u8; MAX_FRAME_SIZE],
    length: usize,
}

impl Frame {
    pub const fn empty() -> Self {
        Self {
            data: [0u8; MAX_FRAME_SIZE],
            length: 0,
        }
    }

    pub const fn from_raw(data: [u8; MAX_FRAME_SIZE], length: usize) -> Self {
        Self { data, length }
    }

    pub const fn as_slice(&self) -> &[u8] {
        let (slice, _) = self.data.split_at(self.length);
        slice
    }

    pub const fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    pub const fn len(&self) -> usize {
        self.length
    }

    pub const fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub const fn capacity(&self) -> usize {
        MAX_FRAME_SIZE
    }

    pub const fn buffer_mut(&mut self) -> &mut [u8; MAX_FRAME_SIZE] {
        &mut self.data
    }

    pub fn set_length(&mut self, length: usize) -> Result<(), FrameError> {
        if length > MAX_FRAME_SIZE {
            return Err(FrameError::TooLong {
                got: length,
                max: MAX_FRAME_SIZE,
            });
        }
        self.length = length;
        Ok(())
    }

    pub fn serialize<T>(value: &T) -> Result<Self, FrameError>
    where
        T: Serialize + ?Sized,
    {
        let mut frame = Self::empty();
        let used = postcard::to_slice(value, &mut frame.data)
            .map_err(FrameError::Serialize)?;
        frame.length = used.len();
        Ok(frame)
    }

    pub fn deserialize<T>(&self) -> Result<T, FrameError>
    where
        T: for<'de> Deserialize<'de>,
    {
        postcard::from_bytes(self.as_slice()).map_err(FrameError::Deserialize)
    }
}
