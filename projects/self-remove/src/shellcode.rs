use core::fmt::{Debug, Display, Formatter};

use heapless::Vec;
use iced_x86::{Code, Decoder, DecoderOptions, Encoder, IcedError, Instruction, MemoryOperand, Register};
use ntapi::ntpsapi::NtSuspendThread;
use toolkit::println;
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
    pub fn try_remove_file(path: usize, duration_ms: u32, max_retries: u64) -> Result<Self, EncoderError> {
        const STACK_FRAME: i32 = 0x100;
        let mut encoder = EncoderWithRip::<N>::new();

        Self::prologue(&mut encoder)?;
        encoder.mov_reg_imm64(Register::R14, path as u64)?;
        encoder.mov_reg_imm64(Register::R12, max_retries)?;

        let retry_start = encoder.new_label();
        let open_success = encoder.new_label();
        let error_exit = encoder.new_label();
        let done = encoder.new_label();
        let retry_loop = encoder.new_label();
        let set_info_retry = encoder.new_label();
        let set_info_retry_loop = encoder.new_label();
        let open_success_continue = encoder.new_label();

        encoder.bind_label(retry_start);

        Self::try_open(&mut encoder, retry_loop, error_exit, open_success)?;

        encoder.bind_label(open_success);
        encoder.bind_label(set_info_retry);

        Self::try_set_delete(&mut encoder, set_info_retry_loop, error_exit, open_success_continue)?;

        encoder.bind_label(set_info_retry_loop);
        Self::handle_retry(&mut encoder, duration_ms, set_info_retry, error_exit)?;

        encoder.bind_label(open_success_continue);
        Self::finalize(&mut encoder, done, error_exit)?;

        encoder.bind_label(retry_loop);
        Self::handle_retry(&mut encoder, duration_ms, retry_start, error_exit)?;

        Self::epilogue(&mut encoder, error_exit, done, STACK_FRAME)?;

        Ok(Self(encoder.into_vec()?))
    }

    fn prologue(encoder: &mut EncoderWithRip<N>) -> Result<(), EncoderError> {
        encoder.sub_rsp(0x100)?;
        encoder.mov_reg_imm64(Register::RAX, 0)?;
        for offset in [0x20, 0x28, 0x30, 0x38, 0x40, 0x48, 0x50, 0x58] {
            encoder.mov_mem_rsp_r64(offset, Register::RAX)?;
        }
        Ok(())
    }

    fn try_open(
        encoder: &mut EncoderWithRip<N>,
        retry_loop: Label,
        error_exit: Label,
        open_success: Label,
    ) -> Result<(), EncoderError> {
        Self::nt_open_file(encoder)?;
        encoder.mov_r64_mem_rsp(Register::R13, 0x20)?;
        encoder.test_rax_rax()?;
        encoder.je(open_success)?;

        let error_codes = [
            (STATUS_OBJECT_NAME_NOT_FOUND, error_exit),
            (STATUS_SHARING_VIOLATION, retry_loop),
            (STATUS_ACCESS_DENIED, retry_loop),
        ];
        for &(code, target) in &error_codes {
            encoder.cmp_eax_imm32(code as _)?;
            encoder.je(target)?;
        }
        encoder.jmp(retry_loop)?;
        Ok(())
    }

    fn try_set_delete(
        encoder: &mut EncoderWithRip<N>,
        retry_loop: Label,
        error_exit: Label,
        success: Label,
    ) -> Result<(), EncoderError> {
        Self::nt_set_information_file(encoder)?;
        encoder.test_rax_rax()?;
        encoder.je(success)?;

        encoder.cmp_eax_imm32(STATUS_CANNOT_DELETE as _)?;
        encoder.je(retry_loop)?;
        encoder.jmp(error_exit)?;
        Ok(())
    }

    fn handle_retry(
        encoder: &mut EncoderWithRip<N>,
        duration_ms: u32,
        target: Label,
        error_exit: Label,
    ) -> Result<(), EncoderError> {
        encoder.dec_test_r12()?;
        encoder.je(error_exit)?;
        Self::sleep_ms(encoder, duration_ms)?;
        encoder.jmp(target)?;
        Ok(())
    }

    fn finalize(
        encoder: &mut EncoderWithRip<N>,
        done: Label,
        error_exit: Label,
    ) -> Result<(), EncoderError> {
        Self::nt_close(encoder)?;
        encoder.test_rax_rax()?;
        encoder.jne(error_exit)?;
        encoder.mov_reg_imm32(Register::EAX, 0)?;
        encoder.jmp(done)?;
        Ok(())
    }

    fn epilogue(
        encoder: &mut EncoderWithRip<N>,
        error_exit: Label,
        done: Label,
        stack_frame: i32,
    ) -> Result<(), EncoderError> {
        encoder.bind_label(error_exit);
        encoder.mov_reg_imm32(Register::EAX, 1)?;

        encoder.bind_label(done);
        encoder.add_rsp(stack_frame)?;
        // encoder.ret()?;
        encoder.mov_reg_imm64(Register::R10, 0xFFFFFFFFFFFFFFFF)?;
        encoder.mov_reg_imm64(Register::RDX, 0)?;
        encoder.syscall(0x2C)?;

        Ok(())
    }

    fn nt_open_file(encoder: &mut EncoderWithRip<N>) -> Result<(), EncoderError> {
        encoder.lea_rsp(Register::R10, 0x20)?;
        encoder.mov_reg_imm64(Register::RDX, (DELETE | SYNCHRONIZE | FILE_READ_ATTRIBUTES) as u64)?;
        encoder.mov_reg_reg(Register::R8, Register::R14)?;
        encoder.lea_rsp(Register::R9, 0x40)?;
        encoder.mov_mem_rsp_imm32(0x28, 0x7)?;
        encoder.mov_mem_rsp_imm32(0x30, 0x60)?;
        encoder.syscall(0x33)?;
        Ok(())
    }

    fn nt_set_information_file(encoder: &mut EncoderWithRip<N>) -> Result<(), EncoderError> {
        encoder.sub_rsp(0x38)?;
        encoder.encode(Instruction::with2(Code::Mov_r64_rm64, Register::R10, Register::R13)?)?;
        encoder.lea_rsp(Register::RDX, 0x10)?;
        encoder.lea_rsp(Register::R8, 0x20)?;
        encoder.mov_reg_imm32(Register::R9D, 1)?;
        encoder.mov_mem_rsp_imm32(0x28, 13)?;
        encoder.mov_mem_rsp_imm32(0x10, 0)?;
        encoder.mov_mem_rsp_imm32(0x18, 0)?;
        encoder.mov_mem_rsp_imm32(0x20, 1)?;
        encoder.syscall(0x27)?;
        encoder.add_rsp(0x38)?;
        Ok(())
    }

    fn sleep_ms(encoder: &mut EncoderWithRip<N>, duration_ms: u32) -> Result<(), EncoderError> {
        let delay = -(duration_ms as i64 * 10_000);
        encoder.sub_rsp(8)?;
        encoder.mov_mem_rsp_imm32(0, delay as i32)?;
        encoder.lea_rsp(Register::RDX, 0)?;
        encoder.mov_reg_imm64(Register::R10, 0)?;
        encoder.syscall(0x34)?;
        encoder.add_rsp(8)?;
        Ok(())
    }

    fn nt_close(encoder: &mut EncoderWithRip<N>) -> Result<(), EncoderError> {
        encoder.encode(Instruction::with2(Code::Mov_r64_rm64, Register::R10, Register::R13)?)?;
        encoder.syscall(0xF)?;
        Ok(())
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