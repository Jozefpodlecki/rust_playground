use core::marker::PhantomData;
use crate::encoder::{registers::*, *};

pub struct Pop<'a, Buf, Fixups, const L: usize>(&'a mut Encoder<Buf, Fixups, L>)
where
    Buf: BufferStorage,
    Fixups: FixupStorage;

impl<'a, Buf, Fixups, const L: usize> Pop<'a, Buf, Fixups, L>
where
    Buf: BufferStorage,
    Fixups: FixupStorage,
{
    pub fn new(encoder: &'a mut Encoder<Buf, Fixups, L>) -> Self {
        Self(encoder)
    }

    #[inline]
    pub fn rax(&mut self) -> &mut Encoder<Buf, Fixups, L> {
        self.0.emit_byte(0x58 + Rax::ENC);
        self.0
    }
    
    #[inline]
    pub fn rbx(&mut self) -> &mut Encoder<Buf, Fixups, L> {
        self.0.emit_byte(0x58 + Rbx::ENC);
        self.0
    }
    
    #[inline]
    pub fn rcx(&mut self) -> &mut Encoder<Buf, Fixups, L> {
        self.0.emit_byte(0x58 + Rcx::ENC);
        self.0
    }
    
    #[inline]
    pub fn rdx(&mut self) -> &mut Encoder<Buf, Fixups, L> {
        self.0.emit_byte(0x58 + Rdx::ENC);
        self.0
    }
    
    #[inline]
    pub fn rsi(&mut self) -> &mut Encoder<Buf, Fixups, L> {
        self.0.emit_byte(0x58 + Rsi::ENC);
        self.0
    }
    
    #[inline]
    pub fn rdi(&mut self) -> &mut Encoder<Buf, Fixups, L> {
        self.0.emit_byte(0x58 + Rdi::ENC);
        self.0
    }
    
    #[inline]
    pub fn rbp(&mut self) -> &mut Encoder<Buf, Fixups, L> {
        self.0.emit_byte(0x58 + Rbp::ENC);
        self.0
    }
    
    #[inline]
    pub fn rsp(&mut self) -> &mut Encoder<Buf, Fixups, L> {
        self.0.emit_byte(0x58 + Rsp::ENC);
        self.0
    }
}

