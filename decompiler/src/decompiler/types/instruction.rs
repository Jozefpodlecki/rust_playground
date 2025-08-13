
use capstone::{arch::{x86::{X86Insn, X86Operand, X86OperandType}, DetailsArchInsn}, Capstone, Insn};
use log::info;
use std::fmt::{self, Display, Formatter};
use crate::decompiler::types::{extract_target, get_2_operands, get_operand, ConditionCode, Register}; 

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OperandSize {
    Byte = 1,
    Word = 2,
    Dword = 4,
    Qword = 8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl Operand {
    pub fn get_immediate(&self) -> u64 {
        match self {
            Operand::Imm(imm) => *imm as u64,
            _ => panic!("Operand is not an immediate"),
        }
    }
}

impl From<X86Operand> for Operand {

    fn from(value: X86Operand) -> Self {
        match &value.op_type {
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
                size: value.size.into(),
                segment: if mem.segment().0 == 0 { None } else { Some(Register::from(mem.segment())) },
            },
            _ => panic!("Invalid operand"),
        }
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionPrefix {
    None = 0,
    Rep = 243
}

impl From<&[u8; 4]> for InstructionPrefix {

    fn from(value: &[u8; 4]) -> Self {
        if value[0] == 243 {
            return InstructionPrefix::Rep
        }

        return InstructionPrefix::None
    }
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub id: X86Insn,
    pub prefix: InstructionPrefix,
    pub kind: InstructionType,
    pub mnemonic: String,
    pub op_str: String,
    pub address: u64,
    pub length: u64,
    pub operands: Vec<X86Operand>
}

#[derive(Debug, Clone, PartialEq)]
pub enum RepeatableInstruction {
    Mov(Operand, Operand),   // MOVSB/MOVSW/MOVSD/MOVSQ
    Stos(Operand),           // STOSB/W/D/Q
    Lods(Operand),           // LODSB/W/D/Q
    Scas(Operand),           // SCASB/W/D/Q
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstructionType {
    Invalid,
    Shl(Operand, Operand),
    Shr(Operand, Operand),
    Push(Operand),
    Pop(Operand),
    Inc(Operand),
    Dec(Operand),
    Cmp(Operand, Operand),
    Mov(Operand, Operand),
    MovZX(Operand, Operand),
    Add(Operand, Operand),
    Adc(Operand, Operand),
    Sub(Operand, Operand),
    Test(Operand, Operand),
    Lea(Operand, Operand),
    Xor(Operand, Operand),
    Rep(RepeatableInstruction),
    Nop,
    Cld,
    Leave,
    Int3,
    ConditionalJump(ConditionCode, u64),
    UnconditionalJump(Operand),
    Call(Operand),
    Ret
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
        let id = insn.id().0;
        let id = X86Insn::from(id);

        if id == X86Insn::X86_INS_INVALID {
            return Instruction::invalid(id, insn);
        }

        let detail = cs.insn_detail(insn).unwrap();
        let arch_detail = detail.arch_detail();
        let x86_detail = arch_detail.x86().unwrap();
        let operands: Vec<_> = x86_detail.operands().collect();
        let mnemonic = insn.mnemonic().unwrap().to_string();
        let op_str = insn.op_str().unwrap().to_string();
        let address = insn.address();
        let length = insn.len() as u64;
        let prefix = x86_detail.prefix().into();

        let kind = classify_instruction(id, prefix, &operands);

        Instruction {
            id,
            prefix,
            kind,
            mnemonic,
            op_str,
            address,
            length,
            operands
        }
    }
}

impl Instruction {
    pub fn invalid(id: X86Insn, insn: &Insn) -> Self {
        Self {
            id,
            kind: InstructionType::Invalid,
            length: insn.len() as u64,
            address: insn.address(),
            mnemonic: insn.mnemonic().unwrap().to_string(),
            op_str: insn.op_str().unwrap().to_string(),
            operands: vec![],
            prefix: InstructionPrefix::None
        }
    }

    pub fn get_jump_target(&self) -> Option<u64> {

        if self.kind == InstructionType::Invalid {
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

pub fn classify_instruction(
    id: X86Insn,
    prefix: InstructionPrefix,
    operands: &[X86Operand],
) -> InstructionType {

    match id {
        X86Insn::X86_INS_MOVZX => {
            let (first_op, second_op) = get_2_operands(operands);
            InstructionType::MovZX(second_op, first_op)
        },
        X86Insn::X86_INS_LEAVE => InstructionType::Leave,
        X86Insn::X86_INS_CLD => InstructionType::Cld,
        X86Insn::X86_INS_XOR => {
            let (first_op, second_op) = get_2_operands(operands);
            InstructionType::Xor(first_op, second_op)
        },
        X86Insn::X86_INS_SHL => {
            let (first_op, second_op) = get_2_operands(operands);
            InstructionType::Shl(first_op, second_op)
        },
        X86Insn::X86_INS_SHR => {
            let (first_op, second_op) = get_2_operands(operands);
            InstructionType::Shr(first_op, second_op)
        },
        X86Insn::X86_INS_TEST => {
            let (first_op, second_op) = get_2_operands(operands);
            InstructionType::Test(second_op, first_op)
        },
        X86Insn::X86_INS_PUSH => {
            let operand = get_operand(operands);
            InstructionType::Push(operand)
        },
        X86Insn::X86_INS_POP => {
            let operand = get_operand(operands);
            InstructionType::Pop(operand)
        },
        X86Insn::X86_INS_CMP => {
            let (first_op, second_op) = get_2_operands(operands);
            InstructionType::Cmp(second_op, first_op)
        }
        X86Insn::X86_INS_MOV => {
            let (first_op, second_op) = get_2_operands(operands);
            InstructionType::Mov(second_op, first_op)
        },
        X86Insn::X86_INS_ADD => {
            let (first_op, second_op) = get_2_operands(operands);
            InstructionType::Add(first_op, second_op)
        },
        X86Insn::X86_INS_ADC => {
            let (first_op, second_op) = get_2_operands(operands);
            InstructionType::Adc(first_op, second_op)
        },
        X86Insn::X86_INS_SUB => {
            let (first_op, second_op) = get_2_operands(operands);
            InstructionType::Sub(first_op, second_op)
        },
        X86Insn::X86_INS_INC => {
            let operand = get_operand(operands);
            InstructionType::Inc(operand)
        },
        X86Insn::X86_INS_DEC => {
            let operand = get_operand(operands);
            InstructionType::Dec(operand)
        },
        X86Insn::X86_INS_MOVSB => {
            let (first_op, second_op) = get_2_operands(operands);
            

            if prefix == InstructionPrefix::Rep {
                let instr = RepeatableInstruction::Mov(first_op, second_op);
                InstructionType::Rep(instr)
            }
            else {
                InstructionType::Invalid
            }
        },
        X86Insn::X86_INS_REP => {
            let (first_op, second_op) = get_2_operands(operands);
            let instr = RepeatableInstruction::Mov(first_op, second_op);
            InstructionType::Rep(instr)
        },
        X86Insn::X86_INS_NOP => InstructionType::Nop,
        X86Insn::X86_INS_INT3 => InstructionType::Int3,
        X86Insn::X86_INS_CALL => InstructionType::Call(get_operand(operands)),
        X86Insn::X86_INS_RET | X86Insn::X86_INS_RETF | X86Insn::X86_INS_RETFQ => InstructionType::Ret,
        X86Insn::X86_INS_JMP => InstructionType::UnconditionalJump(get_operand(operands)),
        X86Insn::X86_INS_JAE
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
            | X86Insn::X86_INS_JS => {
            let cond = id.into();
            let operand = get_operand(operands);
            let target: u64 = operand.get_immediate();
            
            InstructionType::ConditionalJump(cond, target)
        },
        _ => InstructionType::Invalid,
    }
}