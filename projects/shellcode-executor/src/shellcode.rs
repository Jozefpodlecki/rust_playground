use core::fmt::{Debug, Display, Formatter};

use heapless::Vec;
use iced_x86::{Code, Decoder, DecoderOptions, Encoder, IcedError, Instruction, MemoryOperand, Register};
use winapi::{shared::ntstatus::{STATUS_ACCESS_DENIED, STATUS_CANNOT_DELETE, STATUS_OBJECT_NAME_NOT_FOUND, STATUS_SHARING_VIOLATION}, um::winnt::{DELETE, FILE_READ_ATTRIBUTES, SYNCHRONIZE}};

use crate::encoder::{EncoderError, EncoderWithRip, Label};

pub struct Shellcode<const N: usize>(Vec<u8, N>);

impl<const N: usize> Display for Shellcode<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut decoder = Decoder::new(64, &self.0, DecoderOptions::NONE);
        let mut rip = 0u64;
        while decoder.can_decode() {
            let instr = decoder.decode();
            if instr.is_invalid() {
                break;
            }
            writeln!(f, "0x{:X}: {:?}", rip, instr.code())?;
            rip += instr.len() as u64;
        }
        Ok(())
    }
}

type InstrResult = Result<usize, IcedError>;

impl<const N: usize> Shellcode<N> {
    pub fn nt_write_file() -> Result<Self, EncoderError> {
        let mut encoder = EncoderWithRip::<N>::new();

        encoder.sub_rsp(0x100)?;
           
        // Get some handle (just an example)
        encoder.mov_rax_gs_60()?;
        encoder.mov_r64_mem(Register::RCX, Register::RAX, 0x20)?;
        encoder.mov_r64_mem(Register::RCX, Register::RCX, 0x30)?;
        
        encoder.mov_reg_reg(Register::R10, Register::RCX)?;  // FileHandle
        encoder.mov_reg_imm64(Register::RDX, 0)?;            // Event = NULL
        encoder.mov_reg_imm64(Register::R8, 0)?;             // ApcRoutine = NULL
        encoder.mov_reg_imm64(Register::R9, 0)?;             // ApcContext = NULL
        
        // Zero RAX
        encoder.mov_reg_imm64(Register::RAX, 0)?;
        
        // Zero IO_STATUS_BLOCK at [RSP+0x50] (16 bytes)
        encoder.mov_mem_rsp_r64(0x50, Register::RAX)?;
        encoder.mov_mem_rsp_r64(0x58, Register::RAX)?;
        
        // Zero ByteOffset at [RSP+0x70] (8 bytes)
        encoder.mov_mem_rsp_r64(0x70, Register::RAX)?;
        
        // Zero Key at [RSP+0x78] (4 bytes, but we'll zero 8)
        encoder.mov_mem_rsp_r64(0x78, Register::RAX)?;
        
        // 5th arg: IoStatusBlock pointer at [RSP+0x28]
        encoder.lea_rsp(Register::RAX, 0x50)?;
        encoder.mov_mem_rsp_r64(0x28, Register::RAX)?;
        
        // 6th arg: Buffer pointer at [RSP+0x30]
        let data_label = encoder.new_label();
        encoder.lea_label(Register::RAX, data_label)?;
        encoder.mov_mem_rsp_r64(0x30, Register::RAX)?;
        
        // 7th arg: Length at [RSP+0x38] (immediate value)
        encoder.mov_mem_rsp_imm32(0x38, 22)?;
        
        // 8th arg: ByteOffset pointer at [RSP+0x40]
        encoder.lea_rsp(Register::RAX, 0x70)?;
        encoder.mov_mem_rsp_r64(0x40, Register::RAX)?;
        
        // 9th arg: Key pointer at [RSP+0x48]
        encoder.lea_rsp(Register::RAX, 0x78)?;
        encoder.mov_mem_rsp_r64(0x48, Register::RAX)?;
        
        encoder.syscall(0x8)?;
        encoder.add_rsp(0x100)?;
        encoder.ret()?;

        encoder.bind_label(data_label);
        encoder.declare_octa(0x006F00770020006F006C006C00650048)?;
        encoder.declare_octa(0x000000000000000000000064006C0072)?;

        Ok(Self(encoder.into_vec()?))
    }

    pub fn trampoline_to(addr: usize) -> Result<Self, EncoderError> {
        let mut encoder = EncoderWithRip::new();
        encoder.mov_reg_imm64(Register::RAX, addr as _)?;
        encoder.jmp_r64(Register::RAX)?;
        Ok(Self(encoder.into_vec()?))
    }

    pub fn into_inner(self) -> Vec<u8, N> {
        self.0
    }
}