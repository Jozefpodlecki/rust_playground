use std::fmt::{self, Display, Formatter};

use capstone::{arch::{x86::{X86Insn, X86InsnDetail, X86Operand}, DetailsArchInsn}, Insn, InsnDetail, InsnId};

#[derive(Debug, Clone)]
pub struct Instruction {
    pub id: X86Insn,
    pub mnemonic: String,
    pub op_str: String,
    pub address: u64,
    pub length: u64,
    pub operands: Vec<X86Operand>
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

        Instruction {
            id,
            mnemonic,
            op_str,
            address: insn.address(),
            length: insn.len() as u64,
            operands
        }
    }
}

