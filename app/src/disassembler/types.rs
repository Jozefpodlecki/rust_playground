use std::fmt::{self, Display, Formatter};

use capstone::{arch::{x86::{X86Insn, X86InsnDetail, X86Operand, X86OperandType}, DetailsArchInsn}, Insn, InsnDetail, InsnId};

#[derive(Debug, Clone)]
pub struct Instruction {
    pub id: X86Insn,
    pub kind: InstructionType,
    pub is_valid: bool,
    pub mnemonic: String,
    pub op_str: String,
    pub address: u64,
    pub length: u8,
    pub operands: Vec<X86Operand>
}

#[derive(Debug, Clone)]
pub enum InstructionType {
    Unknown,
    Invalid,
    Nop,
    Int3,
    ConditionalJump(u64),
    UnConditionalJump(CallTarget),
    Call(CallTarget),
    Ret
}

#[derive(Debug, Clone)]
pub enum CallTarget {
    Direct(u64),
    Indirect(String),
    Memory{
        base: Option<String>,
        index: Option<(String, u8)>,
        disp: i64,
        segment: Option<String>,
    },
}

impl Display for Instruction {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{:#x}: ", self.address)?;
        write!(fmt, "{} ", self.mnemonic)?;
        write!(fmt, "{}", self.op_str)?;

        Ok(())
    }
}

impl<'a> From<(&Insn<'a>, &X86InsnDetail<'a>)> for Instruction {
    fn from(data: (&Insn<'a>, &X86InsnDetail<'a>)) -> Self {
        let (insn, detail) = data;
        let id = X86Insn::from(insn.id().0);
        let operands: Vec<_> = detail.operands().collect();
        let mnemonic = insn.mnemonic().unwrap().to_string();
        let op_str = insn.op_str().unwrap().to_string();

        let kind = match id {
            X86Insn::X86_INS_NOP => InstructionType::Nop,
            X86Insn::X86_INS_INT3 => InstructionType::Int3,
            X86Insn::X86_INS_CALL => {
                if let Some(target) = extract_call_target(&operands) {
                    InstructionType::Call(target)
                } else {
                    InstructionType::Unknown
                }
            }
            X86Insn::X86_INS_RET | X86Insn::X86_INS_RETF | X86Insn::X86_INS_RETFQ => InstructionType::Ret,
            X86Insn::X86_INS_JMP => {
                if let Some(target) = extract_jump_target(&operands) {
                    InstructionType::UnConditionalJump(target)
                } else {
                    InstructionType::Unknown
                }
            }
            id if is_conditional_jump(id) => {
                if let Some(target_addr) = extract_conditional_jump_target(&operands) {
                    InstructionType::ConditionalJump(target_addr)
                } else {
                    InstructionType::Unknown
                }
            }
            _ => InstructionType::Unknown,
        };

        Instruction {
            id,
            kind,
            is_valid: true,
            mnemonic,
            op_str,
            address: insn.address(),
            length: insn.len() as u8,
            operands
        }
    }
}

impl Instruction {
    pub fn invalid(instr: &Insn) -> Self {
        let mnemonic = instr.mnemonic().unwrap().to_string();
        let op_str = instr.op_str().unwrap().to_string();
        let length = instr.len() as u8;
        let address = instr.address();

        Self {
            id: X86Insn::X86_INS_INVALID,
            kind: InstructionType::Invalid,
            length,
            address,
            is_valid: false,
            mnemonic,
            op_str,
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

fn extract_call_target(operands: &[capstone::arch::x86::X86Operand]) -> Option<CallTarget> {
    use capstone::arch::x86::X86OperandType;

    if operands.len() != 1 {
        return None;
    }

   match &operands[0].op_type {
        X86OperandType::Imm(imm) => Some(CallTarget::Direct(*imm as u64)),
        X86OperandType::Reg(reg) => Some(CallTarget::Indirect(format!("reg: {:?}", reg))),
        X86OperandType::Mem(mem) => Some(CallTarget::Memory {
            base: regid_to_option_string(mem.base()),
            index: {
                let idx = mem.index();
                if idx.0 == 0 {
                    None
                } else {
                    Some((format!("{:?}", idx), mem.scale() as u8))
                }
            },
            disp: mem.disp(),
            segment: regid_to_option_string(mem.segment()),
        }),
        _ => None,
    }
}

fn regid_to_option_string(reg: capstone::RegId) -> Option<String> {
    // Assuming RegId(0) means invalid/no register
    if reg.0 == 0 {
        None
    } else {
        Some(format!("{:?}", reg))
    }
}

fn extract_jump_target(operands: &[capstone::arch::x86::X86Operand]) -> Option<CallTarget> {
    extract_call_target(operands)
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