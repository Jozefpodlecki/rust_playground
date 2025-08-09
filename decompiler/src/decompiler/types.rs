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


pub trait X86InsnExt {
    fn is_conditional_jump(&self) -> bool;
}

impl X86InsnExt for X86Insn {
    fn is_conditional_jump(&self) -> bool {
        matches!(
            *self, X86Insn::X86_INS_JAE
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
}