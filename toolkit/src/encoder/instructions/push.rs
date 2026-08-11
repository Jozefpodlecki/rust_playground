use crate::encoder::{Encoder, EncoderResult, BufferStorage, FixupStorage};
use crate::encoder::registers::*;

pub struct Push<'a, Buf, Fixups, const L: usize>(&'a mut Encoder<Buf, Fixups, L>) where
    Buf: BufferStorage,
    Fixups: FixupStorage;

impl<'a, Buf, Fixups, const L: usize> Push<'a, Buf, Fixups, L>
where
    Buf: BufferStorage,
    Fixups: FixupStorage,
{
    pub fn new(encoder: &'a mut Encoder<Buf, Fixups, L>) -> Self {
        Self(encoder)
    }

    pub fn rax(mut self) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        self.0.emit_byte(0x50 + Rax::ENC)?;
        Ok(self.0)
    }
    
    pub fn rbx(mut self) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        self.0.emit_byte(0x50 + Rbx::ENC)?;
        Ok(self.0)
    }
    
    pub fn rcx(mut self) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        self.0.emit_byte(0x50 + Rcx::ENC)?;
        Ok(self.0)
    }
    
    pub fn rdx(mut self) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        self.0.emit_byte(0x50 + Rdx::ENC)?;
        Ok(self.0)
    }
    
    pub fn rsi(mut self) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        self.0.emit_byte(0x50 + Rsi::ENC)?;
        Ok(self.0)
    }
    
    pub fn rdi(mut self) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        self.0.emit_byte(0x50 + Rdi::ENC)?;
        Ok(self.0)
    }
    
    pub fn rbp(mut self) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        self.0.emit_byte(0x50 + Rbp::ENC)?;
        Ok(self.0)
    }
    
    pub fn rsp(mut self) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        self.0.emit_byte(0x50 + Rsp::ENC)?;
        Ok(self.0)
    }
}