use core::fmt::{self, Write};

use alloc::vec::Vec;
use winapi::um::winnt::LARGE_INTEGER;


#[repr(C)]
pub struct USN_JOURNAL_DATA {
    pub UsnJournalID: u64,      // DWORDLONG
    pub FirstUsn: i64,          // USN
    pub NextUsn: i64,           // USN
    pub LowestValidUsn: i64,    // USN
    pub MaxUsn: i64,            // USN
    pub MaximumSize: u64,       // DWORDLONG
    pub AllocationDelta: u64,   // DWORDLONG
}

#[repr(C)]
pub struct MFT_ENUM_DATA {
    pub StartFileReferenceNumber: u64,  // DWORDLONG
    pub LowUsn: i64,                    // USN
    pub HighUsn: i64,                   // USN
}

#[repr(C)]
pub struct USN_RECORD_V2 {
    pub RecordLength: u32,                      // DWORD
    pub MajorVersion: u16,                      // WORD
    pub MinorVersion: u16,                      // WORD
    pub FileReferenceNumber: u64,               // DWORDLONG
    pub ParentFileReferenceNumber: u64,         // DWORDLONG
    pub Usn: i64,                               // USN
    pub TimeStamp: LARGE_INTEGER,               // LARGE_INTEGER (i64)
    pub Reason: u32,                            // DWORD
    pub SourceInfo: u32,                        // DWORD
    pub SecurityId: u32,                        // DWORD
    pub FileAttributes: u32,                    // DWORD
    pub FileNameLength: u16,                    // WORD
    pub FileNameOffset: u16,                    // WORD
    // FileName follows: WCHAR FileName[1]      // Variable length at offset
}

impl USN_RECORD_V2 {
    pub fn file_name<'a>(&self, buffer: &'a [u8], offset: usize) -> &'a [u16] {
        let name_ptr = unsafe {
            buffer.as_ptr()
                .add(offset + self.FileNameOffset as usize)
                as *const u16
        };
        let name_len = self.FileNameLength as usize / 2;
        unsafe {
            core::slice::from_raw_parts(name_ptr, name_len)
        }
    }
}

pub struct UsnRecord {
    pub file_reference_number: u64,
    pub parent_file_reference_number: u64,
    pub file_name: Vec<u16>,
    pub is_directory: bool,
}


#[derive(Default)]
pub struct Stats {
    pub file_count: u64,
    pub file_size: u64,
}

#[derive(Default)]
pub struct DirectoryNode {
    pub stats: Stats,
    pub children: Vec<AncestorPath>,
    pub parent: Option<AncestorPath>,
}

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct AncestorPath {
    data: [u16; 260],
    len: usize,
}

impl AncestorPath {
    pub fn new(path: &[u16]) -> Self {
        let mut data = [0u16; 260];
        let len = path.len().min(259);
        data[..len].copy_from_slice(&path[..len]);
        Self { data, len }
    }

    pub fn from_slice(slice: &[u16]) -> Self {
        let mut data = [0u16; 260];
        let len = slice.len().min(259);
        data[..len].copy_from_slice(&slice[..len]);
        Self { data, len }
    }

    pub fn as_slice(&self) -> &[u16] {
        &self.data[..self.len]
    }
}

impl fmt::Display for AncestorPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slice = &self.data[..self.len];
        let mut chars = slice.iter().cloned();
        while let Some(Ok(c)) = char::decode_utf16(&mut chars).next() {
            f.write_char(c)?;
        }
        Ok(())
    }
}


#[derive(PartialOrd, Ord, PartialEq, Eq)]
pub struct AncestorPaths {
    buffer: [u16; 260],
    len: usize,
    root_len: usize,
}

impl AncestorPaths {
    pub fn new(path: &[u16]) -> Self {
        let mut buffer = [0u16; 260];
        let len = path.len().min(259);
        buffer[..len].copy_from_slice(&path[..len]);
        
        let root_len = if len >= 4 && buffer[0] == b'\\' as u16 && 
                          buffer[1] == b'?' as u16 && buffer[2] == b'?' as u16 && 
                          buffer[3] == b'\\' as u16 {
            // Find the drive root: \??\C:\
            let mut pos = 4;
            while pos < len && buffer[pos] != b'\\' as u16 {
                pos += 1;
            }
            if pos < len && buffer[pos] == b'\\' as u16 {
                pos + 1 // include the trailing backslash
            } else {
                4 // just \??\
            }
        } else if len >= 3 && buffer[1] == b':' as u16 && buffer[2] == b'\\' as u16 {
            3 // C:\
        } else if len >= 1 && buffer[0] == b'\\' as u16 {
            1 // \
        } else {
            0
        };
        
        Self { buffer, len, root_len }
    }
}

impl Iterator for AncestorPaths {
    type Item = AncestorPath;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len <= self.root_len {
            return None;
        }

        let mut pos = self.len - 1;
        while pos > self.root_len && self.buffer[pos] != b'\\' as u16 {
            pos -= 1;
        }

        if pos <= self.root_len {
            self.len = self.root_len;
            let mut data = [0u16; 260];
            data[..self.root_len].copy_from_slice(&self.buffer[..self.root_len]);
            return Some(AncestorPath { data, len: self.root_len });
        }

        self.len = pos;
        let mut data = [0u16; 260];
        data[..self.len].copy_from_slice(&self.buffer[..self.len]);
        Some(AncestorPath { data, len: self.len })
    }
}


#[repr(align(8))]
pub struct AlignedBuffer<const N: usize>([u8; N]);

impl<const N: usize> AlignedBuffer<N> {
    pub fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }

    pub const fn new() -> Self {
        Self([0u8; N])
    }
    
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.0.as_mut_ptr()
    }
    
    pub fn as_ptr(&self) -> *const u8 {
        self.0.as_ptr()
    }
    
    pub fn len(&self) -> usize {
        N
    }
}