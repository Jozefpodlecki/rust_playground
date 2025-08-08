use capstone::{arch::{self, x86::{X86Insn, X86Operand, X86OperandType, X86Reg}, BuildsCapstone, BuildsCapstoneSyntax, DetailsArchInsn}, Capstone, Insn, InsnGroupType::CS_GRP_JUMP, InsnId, Instructions};
use anyhow::*;
use object::{Object, ObjectSection};
use ringbuffer::{AllocRingBuffer, RingBuffer};

pub struct Disassembler {

}

impl Disassembler {
    pub fn new() -> Self {
        Self {}
    }

    fn get_text_section<'a>(&self, data: &'a [u8]) -> Result<&'a [u8]> {
        let obj_file = object::File::parse(&*data)?;

        let text_section = obj_file
            .sections()
            .filter(|pr| pr.name().unwrap() == "text")
            .next()
            .unwrap();

        let data = text_section.data()?;

        Ok(data)
    }

    pub fn find_function_entries(&self, data: &[u8]) -> Result<Vec<u64>> {
        
        let data = self.get_text_section(data)?;

        let mut cs = Capstone::new()
            .x86()
            .mode(arch::x86::ArchMode::Mode64)
            .syntax(arch::x86::ArchSyntax::Intel)
            .build()?;

        let mut ring_buf = AllocRingBuffer::<(InsnId, u64, Vec<X86Operand>)>::new(5);
        cs.set_skipdata(true)?;
        cs.set_detail(true)?;
        let mut address = 0;
        let count = 1000;
        let mut offset = 0;
        let mut function_prologues = vec![];

        while offset < data.len() {

            if offset >= data.len() {
                break;
            }

            let chunk = &data[offset..];

            let instructions = cs.disasm_count(data, address, count)?;
            
            if instructions.is_empty() {
                break;
            }

            for instruction in instructions.into_iter() {
                let id = instruction.id();
                let address = instruction.address();
                offset += instruction.len();

                let detail = cs.insn_detail(instruction)?;
                let arch_detail = detail.arch_detail();
                let x86_detail = arch_detail.x86().unwrap();
                
                let operands = x86_detail.operands();
                let operands: Vec<_> = operands.into_iter().collect();
                
                ring_buf.enqueue((id, address, operands));
    
                if let Some(addr) = self.detect_function_prologues(&ring_buf) {
                    function_prologues.push(addr);
                }
            }

            // instruction.is
        }

        Ok(function_prologues)
    }

    fn detect_function_prologues(&self, buffer: &AllocRingBuffer<(InsnId, u64, Vec<X86Operand>)>) -> Option<u64> {
        let buffer: Vec<_> = buffer.to_vec();

        if buffer.len() < 2 {
            return None;
        }

        let (id1, addr1, _ops1) = &buffer[buffer.len() - 2];
        let (id2, addr2, ops2) = &buffer[buffer.len() - 1];

        if id1.0 == X86Insn::X86_INS_PUSH as u32
            && id2.0 == X86Insn::X86_INS_MOV as u32
        {
            if ops2.len() == 2 {
                if let (X86OperandType::Reg(dest), X86OperandType::Reg(src)) = (&ops2[0].op_type, &ops2[1].op_type) {
                    if dest.0 == X86Reg::X86_REG_RBP as u16 && src.0 == X86Reg::X86_REG_RSP as u16 {
                        return Some(*addr1);
                    }
                }
            }
        }

        if id1.0 == X86Insn::X86_INS_ENDBR64 as u32 && id2.0 == X86Insn::X86_INS_PUSH as u32 {
            if ops2.len() == 1 {
                if let X86OperandType::Reg(reg) = &ops2[0].op_type {
                    if reg.0 == X86Reg::X86_REG_RBP as u16 {
                        return Some(*addr1);
                    }
                }
            }
        }

        if id1.0 == X86Insn::X86_INS_PUSH as u32 {
            if let Some(ops) = _ops1.get(0) {
                if let X86OperandType::Reg(reg) = ops.op_type {
                    if reg.0 == X86Reg::X86_REG_RBX as u16
                        || reg.0 == X86Reg::X86_REG_R12 as u16
                        || reg.0 == X86Reg::X86_REG_R13 as u16
                        || reg.0 == X86Reg::X86_REG_R14 as u16
                        || reg.0 == X86Reg::X86_REG_R15 as u16
                    {
                        return Some(*addr1);
                    }
                }
            }
        }

        None
    }
}