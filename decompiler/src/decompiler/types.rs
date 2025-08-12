use std::fmt::{self, Display, Formatter};

use capstone::{arch::{x86::{X86Insn, X86InsnDetail, X86Operand, X86OperandType, X86Reg}, DetailsArchInsn}, Capstone, Insn, InsnDetail, InsnId};

#[derive(Debug, Clone)]
pub struct Instruction {
    pub id: X86Insn,
    pub kind: InstructionType,
    pub is_valid: bool,
    pub mnemonic: String,
    pub op_str: String,
    pub address: u64,
    pub length: u64,
    pub operands: Vec<X86Operand>
}

pub const test: u32 = X86Reg::X86_REG_BX;

#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum Register {
    Unknown(u16),
    Rax = 35,
    Eax = 19,
    Rbx = 37,
    Ebx = 21,
    Bx = 8,
    Rcx = 38,
    Ecx = 22,
    Rdx = 40,
    Edx = 24,
    Rip = 41,
    Rsp = 44,
    Rbp = 36,
    Rsi = 43,
    Rdi = 39,
    Edi = 23,
    R8 = 106,
    R9 = 107,
    R10 = 108,
    R11 = 109,
    R12 = 110,
    R13 = 111,
    R14 = 112,
    R15 = 113,
}

impl From<capstone::RegId> for Register {
    fn from(reg: capstone::RegId) -> Self {
        match reg.0 {
            35 => Register::Rax,
            19 => Register::Eax,
            37 => Register::Rbx,
            21 => Register::Ebx,
            8 => Register::Bx,
            38 => Register::Rcx,
            22 => Register::Ecx,
            40 => Register::Rdx,
            24 => Register::Edx,
            41 => Register::Rip,
            44 => Register::Rsp,
            36 => Register::Rbp,
            43 => Register::Rsi,
            39 => Register::Rdi,
            23 => Register::Edi,
            106 => Register::R8,
            107 => Register::R9,
            108 => Register::R10,
            109 => Register::R11,
            110 => Register::R12,
            111 => Register::R13,
            112 => Register::R14,
            113 => Register::R15,
            _ => Register::Unknown(reg.0 as u16),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ConditionCode {
    // signed / unsigned distinction matters
    Overflow,             // JO
    NotOverflow,          // JNO
    Below,                // JB / JNAE (unsigned <)
    AboveOrEqual,         // JAE / JNB (unsigned >=)
    Equal,                // JE / JZ
    NotEqual,             // JNE / JNZ
    BelowOrEqual,         // JBE (unsigned <=)
    Above,                // JA  (unsigned >)
    Sign,                 // JS
    NotSign,              // JNS
    ParityEven,           // JP / JPE
    ParityOdd,            // JNP / JPO
    Less,                 // JL / JNGE (signed <)
    GreaterOrEqual,       // JGE / JNL (signed >=)
    LessOrEqual,          // JLE / JNG (signed <=)
    Greater,              // JG / JNLE (signed >)
    CXZ,                  // JCXZ / JECXZ / JRCXZ
}

#[derive(Debug, Clone)]
pub enum InstructionType {
    Invalid,
    Shr(Operand, Operand),
    Push(Operand),
    Pop(Operand),
    Inc(Operand),
    Dec(Operand),
    Cmp(Operand, Operand),
    Mov(Operand, Operand),
    Add(Operand, Operand),
    Sub(Operand, Operand),
    Test(Operand, Operand),
    Lea(Operand, Operand),
    Xor(Operand, Operand),
    Nop,
    Leave,
    Int3,
    ConditionalJump(ConditionCode, u64),
    UnconditionalJump(Operand),
    Call(Operand),
    Ret
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum OperandSize {
    Byte = 1,
    Word = 2,
    Dword = 4,
    Qword = 8,
}

#[derive(Debug, Clone)]
pub enum Operand {
    Reg(Register),
    Imm(i64),
    Memory {
        base: Option<Register>,
        index: Option<(Register, u8)>,
        disp: i64,
        segment: Option<Register>,
        size: OperandSize,
    },
}

impl From<u8> for OperandSize {

    fn from(value: u8) -> Self {
        match value {
            1 => OperandSize::Byte,
            2 => OperandSize::Word,
            4 => OperandSize::Dword,
            8 => OperandSize::Qword,
            _ => panic!("Invalid operand size"),
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{:#x}: ", self.address)?;
        write!(fmt, "{} ", self.mnemonic)?;
        write!(fmt, "{}", self.op_str)?;

        Ok(())
    }
}

impl<'a> From<(&Insn<'a>, &Capstone)> for Instruction {
    fn from(data: (&Insn<'a>, &Capstone)) -> Self {
        let (insn, cs) = data;
        let id = X86Insn::from(insn.id().0);

        if id == X86Insn::X86_INS_INVALID {
            return Instruction::invalid(insn)
        }

        let detail = cs.insn_detail(insn).unwrap();
        let arch_detail = detail.arch_detail();
        let x86_detail = arch_detail.x86().unwrap();
        let operands: Vec<_> = x86_detail.operands().collect();
        let mnemonic = insn.mnemonic().unwrap().to_string();
        let op_str = insn.op_str().unwrap().to_string();
        let address = insn.address();
        let length = insn.len() as u64;

        let kind = classify_instruction(id, address, length, &operands);

        Instruction {
            id,
            kind,
            is_valid: true,
            mnemonic,
            op_str,
            address,
            length,
            operands
        }
    }
}

impl Instruction {
    pub fn invalid(insn: &Insn) -> Self {
        Self {
            id: X86Insn::X86_INS_INVALID,
            kind: InstructionType::Invalid,
            length: insn.len() as u64,
            address: insn.address(),
            is_valid: false,
            mnemonic: insn.mnemonic().unwrap().to_string(),
            op_str: insn.op_str().unwrap().to_string(),
            operands: vec![]
        }
    }

    pub fn is_conditional_jump(&self) -> bool {
        matches!(
            self.id, X86Insn::X86_INS_JAE
                | X86Insn::X86_INS_JA
                | X86Insn::X86_INS_JBE
                | X86Insn::X86_INS_JB
                | X86Insn::X86_INS_JCXZ
                | X86Insn::X86_INS_JECXZ
                | X86Insn::X86_INS_JE
                | X86Insn::X86_INS_JGE
                | X86Insn::X86_INS_JG
                | X86Insn::X86_INS_JLE
                | X86Insn::X86_INS_JL
                | X86Insn::X86_INS_JNE
                | X86Insn::X86_INS_JNO
                | X86Insn::X86_INS_JNP
                | X86Insn::X86_INS_JNS
                | X86Insn::X86_INS_JO
                | X86Insn::X86_INS_JP
                | X86Insn::X86_INS_JS
        )
    }

    pub fn get_jump_target(&self) -> Option<u64> {

        if !self.is_valid {
            return None;
        }

        if !self.is_conditional_jump() {
            return None;
        }

        if self.operands.len() != 1 {
            return None;
        }

        match self.operands[0].op_type {
            X86OperandType::Imm(offset) => {
                let target = (self.address as i64)
                    .checked_add(self.length as i64)?
                    .checked_add(offset)?;
                Some(target as u64)
            },
            _ => None
        }
    }
}

fn classify_instruction(
    id: X86Insn,
    address: u64,
    length: u64,
    operands: &[X86Operand],
) -> InstructionType {

    match id {
        X86Insn::X86_INS_LEAVE => InstructionType::Leave,
        X86Insn::X86_INS_XOR => {
            let mut operands = capstone_operands_to_internal(operands);
            let mut operands = operands.drain(0..2);
            let first_op = operands.next().unwrap();
            let second_op = operands.next().unwrap();
            InstructionType::Xor(first_op, second_op)
        },
        X86Insn::X86_INS_SHR => {
            let mut operands = capstone_operands_to_internal(operands);
            let mut operands = operands.drain(0..2);
            let first_op = operands.next().unwrap();
            let second_op = operands.next().unwrap();
            InstructionType::Shr(first_op, second_op)
        },
        X86Insn::X86_INS_TEST => {
            let mut operands = capstone_operands_to_internal(operands);
            let mut operands = operands.drain(0..2);
            let first_op = operands.next().unwrap();
            let second_op = operands.next().unwrap();
            InstructionType::Test(second_op, first_op)
        },
        X86Insn::X86_INS_PUSH => {
            let operand = capstone_operands_to_internal(operands).remove(0);
            InstructionType::Push(operand)
        },
        X86Insn::X86_INS_POP => {
            let operand = capstone_operands_to_internal(operands).remove(0);
            InstructionType::Pop(operand)
        },
        X86Insn::X86_INS_CMP => {
            let mut operands = capstone_operands_to_internal(operands);
            let mut operands = operands.drain(0..2);
            let first_op = operands.next().unwrap();
            let second_op = operands.next().unwrap();
            InstructionType::Cmp(second_op, first_op)
        }
        X86Insn::X86_INS_MOV => {
            let mut operands = capstone_operands_to_internal(operands);
            let mut operands = operands.drain(0..2);
            let first_op = operands.next().unwrap();
            let second_op = operands.next().unwrap();
            InstructionType::Mov(second_op, first_op)
        },
        X86Insn::X86_INS_ADD => {
            let mut operands = capstone_operands_to_internal(operands);
            let mut operands = operands.drain(0..2);
            let first_op = operands.next().unwrap();
            let second_op = operands.next().unwrap();
            InstructionType::Add(first_op, second_op)
        },
        X86Insn::X86_INS_SUB => {
            let mut operands = capstone_operands_to_internal(operands);
            let mut operands = operands.drain(0..2);
            let first_op = operands.next().unwrap();
            let second_op = operands.next().unwrap();
            InstructionType::Sub(first_op, second_op)
        },
        X86Insn::X86_INS_INC => {
            let operand = capstone_operands_to_internal(operands).remove(0);
            InstructionType::Inc(operand)
        },
        X86Insn::X86_INS_DEC => {
            let operand = capstone_operands_to_internal(operands).remove(0);
            InstructionType::Dec(operand)
        },
        X86Insn::X86_INS_NOP => InstructionType::Nop,
        X86Insn::X86_INS_INT3 => InstructionType::Int3,
        X86Insn::X86_INS_CALL => extract_target(operands)
            .map(InstructionType::Call)
            .unwrap_or(InstructionType::Invalid),
        X86Insn::X86_INS_RET | X86Insn::X86_INS_RETF | X86Insn::X86_INS_RETFQ => InstructionType::Ret,
        X86Insn::X86_INS_JMP => extract_target(operands)
            .map(InstructionType::UnconditionalJump)
            .unwrap_or(InstructionType::Invalid),
        id if is_conditional_jump(id) => {
            let cond = map_condition_code(id).unwrap();
            let target = extract_conditional_jump_target(&operands).unwrap();
            InstructionType::ConditionalJump(cond, target)
        },
        _ => InstructionType::Invalid,
    }
}

pub fn is_conditional_jump(id: X86Insn) -> bool {
    matches!(
        id, X86Insn::X86_INS_JAE
            | X86Insn::X86_INS_JA
            | X86Insn::X86_INS_JBE
            | X86Insn::X86_INS_JB
            | X86Insn::X86_INS_JCXZ
            | X86Insn::X86_INS_JECXZ
            | X86Insn::X86_INS_JE
            | X86Insn::X86_INS_JGE
            | X86Insn::X86_INS_JG
            | X86Insn::X86_INS_JLE
            | X86Insn::X86_INS_JL
            | X86Insn::X86_INS_JNE
            | X86Insn::X86_INS_JNO
            | X86Insn::X86_INS_JNP
            | X86Insn::X86_INS_JNS
            | X86Insn::X86_INS_JO
            | X86Insn::X86_INS_JP
            | X86Insn::X86_INS_JS
    )
}

fn capstone_operands_to_internal(ops: &[X86Operand]) -> Vec<Operand> {
    ops.iter().map(|op| {
        match &op.op_type {
            X86OperandType::Reg(reg) => Operand::Reg(Register::from(*reg)),
            X86OperandType::Imm(val) => Operand::Imm(*val),
            X86OperandType::Mem(mem) => Operand::Memory {
                base: if mem.base().0 == 0 { None } else { Some(Register::from(mem.base())) },
                index: {
                    let idx = mem.index();
                    if idx.0 == 0 {
                        None
                    } else {
                        Some((Register::from(idx), mem.scale() as u8))
                    }
                },
                disp: mem.disp(),
                size: op.size.into(),
                segment: if mem.segment().0 == 0 { None } else { Some(Register::from(mem.segment())) },
            },
            _ => Operand::Imm(0), // fallback, could consider Option or Result here instead
        }
    }).collect()
}

fn extract_target(operands: &[capstone::arch::x86::X86Operand]) -> Option<Operand> {
    if operands.len() != 1 {
        return None;
    }

    match &operands[0].op_type {
        capstone::arch::x86::X86OperandType::Imm(imm) => Some(Operand::Imm(*imm)),
        capstone::arch::x86::X86OperandType::Reg(reg) => Some(Operand::Reg(Register::from(*reg))),
        capstone::arch::x86::X86OperandType::Mem(mem) => Some(Operand::Memory {
            base: if mem.base().0 == 0 { None } else { Some(Register::from(mem.base())) },
            index: {
                let idx = mem.index();
                if idx.0 == 0 {
                    None
                } else {
                    Some((Register::from(idx), mem.scale() as u8))
                }
            },
            disp: mem.disp(),
            size: operands[0].size.into(),
            segment: if mem.segment().0 == 0 { None } else { Some(Register::from(mem.segment())) },
        }),
        _ => None,
    }
}

fn map_condition_code(id: X86Insn) -> Option<ConditionCode> {
    use ConditionCode::*;
    match id {
        X86Insn::X86_INS_JO  => Some(Overflow),
        X86Insn::X86_INS_JNO => Some(NotOverflow),
        X86Insn::X86_INS_JB => Some(Below),
        X86Insn::X86_INS_JAE => Some(AboveOrEqual),
        X86Insn::X86_INS_JE => Some(Equal),
        X86Insn::X86_INS_JNE => Some(NotEqual),
        X86Insn::X86_INS_JBE => Some(BelowOrEqual),
        X86Insn::X86_INS_JA => Some(Above),
        X86Insn::X86_INS_JS => Some(Sign),
        X86Insn::X86_INS_JNS => Some(NotSign),
        X86Insn::X86_INS_JP  => Some(ParityEven),
        X86Insn::X86_INS_JNP  => Some(ParityOdd),
        X86Insn::X86_INS_JL => Some(Less),
        X86Insn::X86_INS_JGE  => Some(GreaterOrEqual),
        X86Insn::X86_INS_JLE => Some(LessOrEqual),
        X86Insn::X86_INS_JG => Some(Greater),
        X86Insn::X86_INS_JCXZ  |
        X86Insn::X86_INS_JECXZ |
        X86Insn::X86_INS_JRCXZ => Some(CXZ),
        _ => None,
    }
}

fn extract_conditional_jump_target(operands: &[capstone::arch::x86::X86Operand]) -> Option<u64> {
    use capstone::arch::x86::X86OperandType;

    if operands.len() != 1 {
        return None;
    }

    if let X86OperandType::Imm(imm) = operands[0].op_type {
        Some(imm as u64)
    } else {
        None
    }
}