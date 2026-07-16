use core::sync::atomic::AtomicU64;

#[repr(u64)]
#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    Add = 0,
    Sub = 1,
    Mul = 2,
    Div = 3,
    Custom(u64) = 4,
}

impl Default for OpCode {
    fn default() -> Self {
        OpCode::Add
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Context {
    pub registers: [u64; 8],
    pub flags: u64,
    pub pc: u64,
    pub stack_ptr: u64,
    pub op_history: [OpCode; 16],
    pub history_index: u64,
    pub execution_count: AtomicU64,
    pub data_section: [u64; 32],
    pub heap_ptr: u64,
    pub heap_size: u64,
    pub cycles: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

#[derive(Debug)]
#[repr(C)]
pub struct ExecutionResult {
    pub return_value: u64,
    pub exit_code: u32,
    pub cycles_used: u64,
    pub memory_accessed: u64,
    pub opcode_executed: u64,
    pub success: bool,
    pub error_msg: *const u8,
}