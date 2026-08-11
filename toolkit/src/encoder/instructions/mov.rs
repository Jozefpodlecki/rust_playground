use core::marker::PhantomData;
use crate::encoder::{registers::*, *};

pub struct Mov<'a, Buf, Fixups, const L: usize>(
    &'a mut Encoder<Buf, Fixups, L>,
) where
    Buf: BufferStorage,
    Fixups: FixupStorage;

impl<'a, Buf, Fixups, const L: usize> Mov<'a, Buf, Fixups, L>
where
    Buf: BufferStorage,
    Fixups: FixupStorage,
{
    pub fn new(encoder: &'a mut Encoder<Buf, Fixups, L>) -> Self {
        Self(encoder)
    }

    pub fn rax(self) -> MovDst<'a, Buf, Fixups, L, Rax> {
        MovDst(self.0, PhantomData)
    }
    
    pub fn rbx(self) -> MovDst<'a, Buf, Fixups, L, Rbx> {
        MovDst(self.0, PhantomData)
    }
    
    pub fn rcx(self) -> MovDst<'a, Buf, Fixups, L, Rcx> {
        MovDst(self.0, PhantomData)
    }
    
    pub fn rdx(self) -> MovDst<'a, Buf, Fixups, L, Rdx> {
        MovDst(self.0, PhantomData)
    }
    
    pub fn rsi(self) -> MovDst<'a, Buf, Fixups, L, Rsi> {
        MovDst(self.0, PhantomData)
    }
    
    pub fn rdi(self) -> MovDst<'a, Buf, Fixups, L, Rdi> {
        MovDst(self.0, PhantomData)
    }
    
    pub fn rbp(self) -> MovDst<'a, Buf, Fixups, L, Rbp> {
        MovDst(self.0, PhantomData)
    }
    
    pub fn rsp(self) -> MovDst<'a, Buf, Fixups, L, Rsp> {
        MovDst(self.0, PhantomData)
    }
}

pub struct MovDst<'a, Buf, Fixups, const L: usize, D: Reg>(
    &'a mut Encoder<Buf, Fixups, L>,
    PhantomData<D>,
) where
    Buf: BufferStorage,
    Fixups: FixupStorage;

impl<'a, Buf, Fixups, const L: usize, D: Reg> MovDst<'a, Buf, Fixups, L, D>
where
    Buf: BufferStorage,
    Fixups: FixupStorage,
{
    fn emit_mov_rr<S: Reg>(&mut self) -> EncoderResult<()> {
        let rex = 0x40 | D::REX | S::REX;
        if rex != 0x40 {
            self.0.emit_byte(rex)?;
        }
        self.0.emit_byte(0x89)?;
        self.0.emit_byte(0xC0 | (D::ENC << 3) | S::ENC)?;
        Ok(())
    }
    
    pub fn rbx(mut self) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        self.emit_mov_rr::<Rbx>()?;
        Ok(self.0)
    }
    
    pub fn rcx(mut self) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        self.emit_mov_rr::<Rcx>()?;
        Ok(self.0)
    }
    
    pub fn rdx(mut self) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        self.emit_mov_rr::<Rdx>()?;
        Ok(self.0)
    }
    
    pub fn rsi(mut self) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        self.emit_mov_rr::<Rsi>()?;
        Ok(self.0)
    }
    
    pub fn rdi(mut self) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        self.emit_mov_rr::<Rdi>()?;
        Ok(self.0)
    }
    
    pub fn rbp(mut self) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        self.emit_mov_rr::<Rbp>()?;
        Ok(self.0)
    }
    
    pub fn rsp(mut self) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        self.emit_mov_rr::<Rsp>()?;
        Ok(self.0)
    }
    
    pub fn imm64(mut self, val: u64) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        let rex = 0x48 | D::REX;
        if rex != 0x48 {
            self.0.emit_byte(rex)?;
        }
        self.0.emit_byte(0xB8 + D::ENC)?;
        self.0.emit(&val.to_le_bytes())?;
        Ok(self.0)
    }
    
    pub fn imm32(mut self, val: u32) -> EncoderResult<&'a mut Encoder<Buf, Fixups, L>> {
        let rex = 0x40 | D::REX;
        if rex != 0x40 {
            self.0.emit_byte(rex)?;
        }
        self.0.emit_byte(0xB8 + D::ENC)?;
        self.0.emit(&val.to_le_bytes())?;
        Ok(self.0)
    }
}