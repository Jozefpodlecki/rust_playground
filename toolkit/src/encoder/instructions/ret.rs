use crate::encoder::*;

pub struct Ret<'a, Buf, Fixups, const L: usize>(&'a mut Encoder<Buf, Fixups, L>)
where
    Buf: BufferStorage,
    Fixups: FixupStorage;


impl<'a, Buf, Fixups, const L: usize> Ret<'a, Buf, Fixups, L>
where
    Buf: BufferStorage,
    Fixups: FixupStorage,
{
    pub fn new(encoder: &'a mut Encoder<Buf, Fixups, L>) -> Self {
        Self(encoder)
    }
    
    pub fn near(&mut self) -> EncoderResult<&mut Encoder<Buf, Fixups, L>> {
        self.0.emit_byte(0xC3)?;
        Ok(self.0)
    }
    
    pub fn far(&mut self) -> EncoderResult<&mut Encoder<Buf, Fixups, L>> {
        self.0.emit_byte(0xCB)?;
        Ok(self.0)
    }
}