use bincode::{Decode, Encode};

#[derive(Debug, Decode, Encode, Clone)]
pub struct ProcessModule {
    pub file_path: String,
    pub file_name: String,
    pub entry_point: u64,
    pub size: u32,
    pub base: u64,
}

#[derive(Debug, Decode, Encode, Clone)]
pub struct MemoryBlock {
    pub size: u64,
    pub base: u64,
    pub state: u32,
    pub protect: u32,
    pub module: Option<ProcessModule>,
    pub is_readable: bool,
    pub is_executable: bool,
}
