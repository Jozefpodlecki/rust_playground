pub trait BufferStorage {
    type Bytes: AsRef<[u8]> + AsMut<[u8]>;
    fn push(&mut self, byte: u8) -> EncoderResult<()>;
    fn extend(&mut self, bytes: &[u8]) -> EncoderResult<()>;
    fn len(&self) -> usize;
    fn capacity(&self) -> usize;
    fn clear(&mut self);
    fn as_slice(&self) -> &[u8];
    fn as_mut_slice(&mut self) -> &mut [u8]; 
}

pub trait FixupStorage {
    type Fixups;
    fn push(&mut self, fixup: Fixup) -> EncoderResult<()>;
    fn len(&self) -> usize;
    fn iter(&self) -> core::slice::Iter<'_, Fixup>;
    fn clear(&mut self);
}

pub struct Fixup {
    pub pos: usize,
    pub label: LabelId,
    pub kind: FixupKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LabelId(pub u32);

pub enum FixupKind {
    Rel8,
    Rel32,
    Abs32,
    Abs64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncoderError {
    /// A label was referenced but never defined
    UndefinedLabel(LabelId),
    /// An 8-bit relative offset was out of range (-128 to 127)
    Rel8OutOfRange { pos: usize, offset: isize },
    /// Buffer capacity was exceeded (fixed-size buffer)
    BufferOverflow { capacity: usize, needed: usize },
    /// Fixup storage capacity was exceeded (fixed-size)
    FixupOverflow { capacity: usize, needed: usize },
    /// Label map capacity was exceeded
    LabelMapOverflow { capacity: usize },
    /// Invalid opcode for the given operands
    InvalidOpcode { opcode: u8, operands: OperandKind },
    /// Invalid register combination
    InvalidRegisterCombination { dst: u8, src: u8 },
    /// Invalid immediate value (e.g., out of range for the instruction)
    InvalidImmediate { min: i64, max: i64, value: i64 },
    /// Invalid memory addressing mode
    InvalidAddressing { mode: AddressingMode },
    /// Prefix cannot be used with this instruction
    InvalidPrefix { prefix: u8, opcode: u8 },
    /// Instruction requires REX prefix but it's not available
    RexRequired { opcode: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperandKind {
    Register { reg: u8 },
    Immediate { size: u8, value: u64 },
    Memory { base: u8, index: Option<u8>, scale: u8, disp: i32 },
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressingMode {
    RipRelative { offset: i32 },
    BaseOffset { base: u8, offset: i32 },
    BaseIndex { base: u8, index: u8, scale: u8 },
    BaseIndexOffset { base: u8, index: u8, scale: u8, offset: i32 },
    Absolute { address: u64 },
}

impl core::fmt::Display for EncoderError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            EncoderError::UndefinedLabel(id) => {
                write!(f, "Undefined label: {:?}", id)
            }
            EncoderError::Rel8OutOfRange { pos, offset } => {
                write!(f, "Rel8 offset out of range at position {}: {}", pos, offset)
            }
            EncoderError::BufferOverflow { capacity, needed } => {
                write!(f, "Buffer capacity exceeded: capacity={}, needed={}", capacity, needed)
            }
            EncoderError::FixupOverflow { capacity, needed } => {
                write!(f, "Fixup storage capacity exceeded: capacity={}, needed={}", capacity, needed)
            }
            EncoderError::LabelMapOverflow { capacity } => {
                write!(f, "Label map capacity exceeded: capacity={}", capacity)
            }
            EncoderError::InvalidOpcode { opcode, operands } => {
                write!(f, "Invalid opcode 0x{:02X} for operands: {:?}", opcode, operands)
            }
            EncoderError::InvalidRegisterCombination { dst, src } => {
                write!(f, "Invalid register combination: dst={}, src={}", dst, src)
            }
            EncoderError::InvalidImmediate { min, max, value } => {
                write!(f, "Invalid immediate value: {} (must be between {} and {})", value, min, max)
            }
            EncoderError::InvalidAddressing { mode } => {
                write!(f, "Invalid addressing mode: {:?}", mode)
            }
            EncoderError::InvalidPrefix { prefix, opcode } => {
                write!(f, "Prefix 0x{:02X} cannot be used with opcode 0x{:02X}", prefix, opcode)
            }
            EncoderError::RexRequired { opcode } => {
                write!(f, "REX prefix required for opcode 0x{:02X}", opcode)
            }
        }
    }
}

impl core::error::Error for EncoderError {}

pub type EncoderResult<T> = Result<T, EncoderError>;