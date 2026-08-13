use crate::encoder::{Encoder, EncoderResult, BufferStorage, FixupStorage, LabelId, FixupKind};
use crate::encoder::registers::*;

pub struct Jmp<'a, Buf, Fixups, const L: usize>(
    &'a mut Encoder<Buf, Fixups, L>,
) where
    Buf: BufferStorage,
    Fixups: FixupStorage;

impl<'a, Buf, Fixups, const L: usize> Jmp<'a, Buf, Fixups, L>
where
    Buf: BufferStorage,
    Fixups: FixupStorage,
{
    pub fn new(encoder: &'a mut Encoder<Buf, Fixups, L>) -> Self {
        Self(encoder)
    }

    fn emit_jmp_reg(&mut self, reg_enc: u8, rex: u8) -> EncoderResult<()> {
        if rex != 0 {
            self.0.emit_byte(rex)?;
        }
        self.0.emit_byte(0xFF)?;
        self.0.emit_byte(0xE0 | reg_enc)?;
        Ok(())
    }

    pub fn rax(mut self) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        self.emit_jmp_reg(0, 0)?;
        Ok(self.0)
    }

    pub fn rbx(mut self) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        self.emit_jmp_reg(3, 0)?;
        Ok(self.0)
    }

    pub fn rcx(mut self) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        self.emit_jmp_reg(1, 0)?;
        Ok(self.0)
    }

    pub fn rdx(mut self) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        self.emit_jmp_reg(2, 0)?;
        Ok(self.0)
    }

    pub fn rsi(mut self) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        self.emit_jmp_reg(6, 0)?;
        Ok(self.0)
    }

    pub fn rdi(mut self) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        self.emit_jmp_reg(7, 0)?;
        Ok(self.0)
    }

    pub fn rbp(mut self) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        self.emit_jmp_reg(5, 0)?;
        Ok(self.0)
    }

    pub fn rsp(mut self) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        self.emit_jmp_reg(4, 0)?;
        Ok(self.0)
    }

    pub fn r8(mut self) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        self.emit_jmp_reg(0, 0x41)?;
        Ok(self.0)
    }

    pub fn r9(mut self) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        self.emit_jmp_reg(1, 0x41)?;
        Ok(self.0)
    }

    pub fn r10(mut self) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        self.emit_jmp_reg(2, 0x41)?;
        Ok(self.0)
    }

    pub fn r11(mut self) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        self.emit_jmp_reg(3, 0x41)?;
        Ok(self.0)
    }

    pub fn r12(mut self) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        self.emit_jmp_reg(4, 0x41)?;
        Ok(self.0)
    }

    pub fn r13(mut self) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        self.emit_jmp_reg(5, 0x41)?;
        Ok(self.0)
    }

    pub fn r14(mut self) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        self.emit_jmp_reg(6, 0x41)?;
        Ok(self.0)
    }

    pub fn r15(mut self) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        self.emit_jmp_reg(7, 0x41)?;
        Ok(self.0)
    }

    pub fn label(mut self, label: LabelId) -> &'a mut Encoder<Buf, Fixups, L> {
        self.0.emit(&[0xE9]);
        let pos = self.0.len();
        self.0.emit(&[0; 4]);
        self.0.add_fixup(pos, label, FixupKind::Rel32);
        self.0
    }

    pub fn short(mut self, label: LabelId) -> &'a mut Encoder<Buf, Fixups, L> {
        self.0.emit(&[0xEB]);
        let pos = self.0.len();
        self.0.emit(&[0]);
        self.0.add_fixup(pos, label, FixupKind::Rel8);
        self.0
    }

    pub fn je(mut self, label: LabelId) -> &'a mut Encoder<Buf, Fixups, L> {
        self.0.emit(&[0x0F, 0x84]);
        let pos = self.0.len();
        self.0.emit(&[0; 4]);
        self.0.add_fixup(pos, label, FixupKind::Rel32);
        self.0
    }

    pub fn jne(mut self, label: LabelId) -> &'a mut Encoder<Buf, Fixups, L> {
        self.0.emit(&[0x0F, 0x85]);
        let pos = self.0.len();
        self.0.emit(&[0; 4]);
        self.0.add_fixup(pos, label, FixupKind::Rel32);
        self.0
    }

    pub fn jz(mut self, label: LabelId) -> &'a mut Encoder<Buf, Fixups, L> {
        self.je(label)
    }

    pub fn jnz(mut self, label: LabelId) -> &'a mut Encoder<Buf, Fixups, L> {
        self.jne(label)
    }

    pub fn jl(mut self, label: LabelId) -> &'a mut Encoder<Buf, Fixups, L> {
        self.0.emit(&[0x0F, 0x8C]);
        let pos = self.0.len();
        self.0.emit(&[0; 4]);
        self.0.add_fixup(pos, label, FixupKind::Rel32);
        self.0
    }

    pub fn jle(mut self, label: LabelId) -> &'a mut Encoder<Buf, Fixups, L> {
        self.0.emit(&[0x0F, 0x8E]);
        let pos = self.0.len();
        self.0.emit(&[0; 4]);
        self.0.add_fixup(pos, label, FixupKind::Rel32);
        self.0
    }

    pub fn jg(mut self, label: LabelId) -> &'a mut Encoder<Buf, Fixups, L> {
        self.0.emit(&[0x0F, 0x8F]);
        let pos = self.0.len();
        self.0.emit(&[0; 4]);
        self.0.add_fixup(pos, label, FixupKind::Rel32);
        self.0
    }

    pub fn jge(mut self, label: LabelId) -> &'a mut Encoder<Buf, Fixups, L> {
        self.0.emit(&[0x0F, 0x8D]);
        let pos = self.0.len();
        self.0.emit(&[0; 4]);
        self.0.add_fixup(pos, label, FixupKind::Rel32);
        self.0
    }

    pub fn jb(mut self, label: LabelId) -> &'a mut Encoder<Buf, Fixups, L> {
        self.0.emit(&[0x0F, 0x82]);
        let pos = self.0.len();
        self.0.emit(&[0; 4]);
        self.0.add_fixup(pos, label, FixupKind::Rel32);
        self.0
    }

    pub fn jbe(mut self, label: LabelId) -> &'a mut Encoder<Buf, Fixups, L> {
        self.0.emit(&[0x0F, 0x86]);
        let pos = self.0.len();
        self.0.emit(&[0; 4]);
        self.0.add_fixup(pos, label, FixupKind::Rel32);
        self.0
    }

    pub fn ja(mut self, label: LabelId) -> &'a mut Encoder<Buf, Fixups, L> {
        self.0.emit(&[0x0F, 0x87]);
        let pos = self.0.len();
        self.0.emit(&[0; 4]);
        self.0.add_fixup(pos, label, FixupKind::Rel32);
        self.0
    }

    pub fn jae(mut self, label: LabelId) -> &'a mut Encoder<Buf, Fixups, L> {
        self.0.emit(&[0x0F, 0x83]);
        let pos = self.0.len();
        self.0.emit(&[0; 4]);
        self.0.add_fixup(pos, label, FixupKind::Rel32);
        self.0
    }
}