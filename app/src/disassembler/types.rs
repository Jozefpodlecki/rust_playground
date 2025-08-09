use capstone::{arch::{x86::{X86Insn, X86InsnDetail, X86Operand}, DetailsArchInsn}, Insn, InsnDetail, InsnId};

#[derive(Debug, Clone)]
pub struct Instruction {
    pub id: X86Insn,
    pub address: u64,
    pub length: u64,
    pub operands: Vec<X86Operand>
}


impl<'a> From<(&Insn<'a>, &X86InsnDetail<'a>)> for Instruction {
    fn from(data: (&Insn<'a>, &X86InsnDetail<'a>)) -> Self {
        let (insn, detail) = data;
        let id = X86Insn::from(insn.id().0);
        let operands: Vec<_> = detail.operands().collect();

        Instruction {
            id,
            address: insn.address(),
            length: insn.len() as u64,
            operands
        }
    }
}

