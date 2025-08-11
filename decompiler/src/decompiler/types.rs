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
    pub length: u64,
    pub operands: Vec<X86Operand>
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
    Push(Operand),
    Mov(Operand, Operand),
    Add(Operand, Operand),
    Sub(Operand, Operand),
    Test(Operand, Operand),
    Lea(Operand, Operand),
    Nop,
    Int3,
    ConditionalJump(ConditionCode, u64),
    UnconditionalJump(CallTarget),
    Call(CallTarget),
    Ret
}

#[derive(Debug, Clone)]
pub enum Operand {
    Reg(String),
    Imm(i64),
    Memory {
        base: Option<String>,
        index: Option<(String, u8)>,
        disp: i64,
        segment: Option<String>,
    },
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
    pub fn invalid(mnemonic: &str, address: u64, length: u64) -> Self {
        Self {
            id: X86Insn::X86_INS_INVALID,
            kind: InstructionType::Invalid,
            length,
            address,
            is_valid: false,
            mnemonic: mnemonic.into(),
            op_str: "".into(),
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

fn take_two_operands(operands: &mut Vec<Operand>) -> Option<(Operand, Operand)> {
    if operands.len() < 2 {
        return None;
    }

    let first = operands.remove(0);
    let second = operands.remove(0);
    Some((first, second))
}

fn classify_instruction(
    id: X86Insn,
    address: u64,
    length: u64,
    operands: &[X86Operand],
) -> InstructionType {
    use X86Insn::*;
    // let operands = capstone_operands_to_internal(&detail.operands().collect::<Vec<_>>());

    match id {
        X86_INS_PUSH => {
            let operand = capstone_operands_to_internal(operands).remove(0);
            InstructionType::Push(operand)
        },
        X86_INS_MOV => {
            let mut operands = capstone_operands_to_internal(operands);
            let mut operands = operands.drain(0..2);
            let first_op = operands.next().unwrap();
            let second_op = operands.next().unwrap();
            InstructionType::Mov(first_op, second_op)
        },
        X86_INS_ADD => {
            let mut operands = capstone_operands_to_internal(operands);
            let mut operands = operands.drain(0..2);
            let first_op = operands.next().unwrap();
            let second_op = operands.next().unwrap();
            InstructionType::Add(first_op, second_op)
        },
        X86_INS_SUB => {
            let mut operands = capstone_operands_to_internal(operands);
            let mut operands = operands.drain(0..2);
            let first_op = operands.next().unwrap();
            let second_op = operands.next().unwrap();
            InstructionType::Sub(first_op, second_op)
        },
        X86_INS_NOP => InstructionType::Nop,
        X86_INS_INT3 => InstructionType::Int3,
        X86_INS_CALL => extract_target(operands)
            .map(InstructionType::Call)
            .unwrap_or(InstructionType::Invalid),
        X86_INS_RET | X86_INS_RETF | X86_INS_RETFQ => InstructionType::Ret,
        X86_INS_JMP => extract_target(operands)
            .map(InstructionType::UnconditionalJump)
            .unwrap_or(InstructionType::Invalid),
        // _ if is_conditional_jump(id) => match extract_conditional_jump_target(address, length, operands) {
        //     Some(target) => InstructionType::ConditionalJump(target),
        //     None => InstructionType::Invalid,
        // },
        // id if is_conditional_jump(id) => {
        //     let cond = map_condition_code(id).unwrap();
        //     let target = extract_conditional_jump_target(&operands) {
        //     InstructionType::ConditionalJump(cond, target)
        // },
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
            X86OperandType::Reg(reg) => Operand::Reg(format!("{:?}", reg)),
            X86OperandType::Imm(val) => Operand::Imm(*val),
            X86OperandType::Mem(mem) => Operand::Memory {
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
            },
            _ => Operand::Imm(0), // fallback for unexpected cases
        }
    }).collect()
}

fn extract_target(operands: &[capstone::arch::x86::X86Operand]) -> Option<CallTarget> {
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

fn regid_to_option_string(reg: capstone::RegId) -> Option<String> {
    // Assuming RegId(0) means invalid/no register
    if reg.0 == 0 {
        None
    } else {
        Some(format!("{:?}", reg))
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