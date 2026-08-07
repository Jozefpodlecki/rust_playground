
use core::{hash::BuildHasherDefault, marker::PhantomData};

use heapless::{IndexMap, String, Vec, index_map::FnvIndexMap};

use crate::encoder::types::*;

mod types;

pub trait BufferStorage {
    type Bytes: AsRef<[u8]> + AsMut<[u8]>;
    fn push(&mut self, byte: u8) -> EncoderResult<()>;
    fn extend(&mut self, bytes: &[u8]) -> EncoderResult<()>;
    fn len(&self) -> usize;
    fn capacity(&self) -> usize;
    fn clear(&mut self);
    fn as_slice(&self) -> &[u8];
    fn as_mut_slice(&mut self) -> &mut [u8]; 
}

pub trait FixupStorage {
    type Fixups;
    fn push(&mut self, fixup: Fixup) -> EncoderResult<()>;
    fn len(&self) -> usize;
    fn iter(&self) -> core::slice::Iter<'_, Fixup>;
    fn clear(&mut self);
}

impl<const N: usize> BufferStorage for Vec<u8, N> {
    type Bytes = [u8; N];
    
    fn push(&mut self, byte: u8) -> EncoderResult<()> {
        self.push(byte).map_err(|_| EncoderError::BufferOverflow {
            capacity: N,
            needed: self.len() + 1,
        })
    }
    
    fn extend(&mut self, bytes: &[u8]) -> EncoderResult<()> {
        self.extend_from_slice(bytes).map_err(|_| EncoderError::BufferOverflow {
            capacity: N,
            needed: self.len() + bytes.len(),
        })
    }

    #[allow(unconditional_recursion)]
    fn len(&self) -> usize {
        Vec::<u8, N>::len(&self)
    }
    
    fn capacity(&self) -> usize {
        self.capacity()
    }
    
    fn clear(&mut self) {
        self.clear()
    }
    
    fn as_slice(&self) -> &[u8] {
        self.as_slice()
    }
    
    fn as_mut_slice(&mut self) -> &mut [u8] {
        self.as_mut_slice()
    }
}

impl<const N: usize> FixupStorage for Vec<Fixup, N> {
    type Fixups = [Fixup; N];
    
    fn push(&mut self, fixup: Fixup) -> EncoderResult<()> {
        self.push(fixup).map_err(|_| EncoderError::FixupOverflow {
            capacity: N,
            needed: self.len() + 1,
        })
    }

    #[allow(unconditional_recursion)]
    fn len(&self) -> usize {
       Vec::<Fixup, N>::len(&self)
    }
    
    fn iter(&self) -> core::slice::Iter<'_, Fixup> {
        self.as_slice().iter()
    }
    
    fn clear(&mut self) {
        self.clear()
    }
}

pub trait Reg {
    const ENC: u8;
    const REX: u8;
}

macro_rules! regs {
    ($($reg:ident => $enc:expr),* $(,)?) => {
        $(
            pub struct $reg;
            impl Reg for $reg {
                const ENC: u8 = $enc;
                const REX: u8 = if $enc >= 8 { 0x01 } else { 0 };
            }
        )*
    };
}

regs! {
    Rax => 0, Rbx => 3, Rcx => 1, Rdx => 2,
    Rsi => 6, Rdi => 7, Rbp => 5, Rsp => 4,
    R8 => 8, R9 => 9, R10 => 10, R11 => 11,
    R12 => 12, R13 => 13, R14 => 14, R15 => 15,
}

pub struct Push<'a, Buf, Fixups, const L: usize> 
where
    Buf: BufferStorage,
    Fixups: FixupStorage,
{
    encoder: &'a mut Encoder<Buf, Fixups, L>,
}

impl<'a, Buf, Fixups, const L: usize> Push<'a, Buf, Fixups, L>
where
    Buf: BufferStorage,
    Fixups: FixupStorage,
{
    pub fn rax(&mut self) -> EncoderResult<&mut Encoder<Buf, Fixups, L>> {
        self.encoder.emit_byte(0x50 + Rax::ENC);
        Ok(self.encoder)
    }
    
    pub fn rbx(&mut self) -> &mut Encoder<Buf, Fixups, L> {
        self.encoder.emit_byte(0x50 + Rbx::ENC);
        self.encoder
    }
    
    pub fn rcx(&mut self) -> &mut Encoder<Buf, Fixups, L> {
        self.encoder.emit_byte(0x50 + Rcx::ENC);
        self.encoder
    }
    
    pub fn rdx(&mut self) -> &mut Encoder<Buf, Fixups, L> {
        self.encoder.emit_byte(0x50 + Rdx::ENC);
        self.encoder
    }
    
    pub fn rsi(&mut self) -> &mut Encoder<Buf, Fixups, L> {
        self.encoder.emit_byte(0x50 + Rsi::ENC);
        self.encoder
    }
    
    pub fn rdi(&mut self) -> &mut Encoder<Buf, Fixups, L> {
        self.encoder.emit_byte(0x50 + Rdi::ENC);
        self.encoder
    }
    
    pub fn rbp(&mut self) -> &mut Encoder<Buf, Fixups, L> {
        self.encoder.emit_byte(0x50 + Rbp::ENC);
        self.encoder
    }
    
    pub fn rsp(&mut self) -> &mut Encoder<Buf, Fixups, L> {
        self.encoder.emit_byte(0x50 + Rsp::ENC);
        self.encoder
    }
}

pub struct Pop<'a, Buf, Fixups, const L: usize> 
where
    Buf: BufferStorage,
    Fixups: FixupStorage,
{
    encoder: &'a mut Encoder<Buf, Fixups, L>,
}

impl<'a, Buf, Fixups, const L: usize> Pop<'a, Buf, Fixups, L>
where
    Buf: BufferStorage,
    Fixups: FixupStorage,
{
    pub fn rax(&mut self) -> &mut Encoder<Buf, Fixups, L> {
        self.encoder.emit_byte(0x58 + Rax::ENC);
        self.encoder
    }
    
    pub fn rbx(&mut self) -> &mut Encoder<Buf, Fixups, L> {
        self.encoder.emit_byte(0x58 + Rbx::ENC);
        self.encoder
    }
    
    pub fn rcx(&mut self) -> &mut Encoder<Buf, Fixups, L> {
        self.encoder.emit_byte(0x58 + Rcx::ENC);
        self.encoder
    }
    
    pub fn rdx(&mut self) -> &mut Encoder<Buf, Fixups, L> {
        self.encoder.emit_byte(0x58 + Rdx::ENC);
        self.encoder
    }
    
    pub fn rsi(&mut self) -> &mut Encoder<Buf, Fixups, L> {
        self.encoder.emit_byte(0x58 + Rsi::ENC);
        self.encoder
    }
    
    pub fn rdi(&mut self) -> &mut Encoder<Buf, Fixups, L> {
        self.encoder.emit_byte(0x58 + Rdi::ENC);
        self.encoder
    }
    
    pub fn rbp(&mut self) -> &mut Encoder<Buf, Fixups, L> {
        self.encoder.emit_byte(0x58 + Rbp::ENC);
        self.encoder
    }
    
    pub fn rsp(&mut self) -> &mut Encoder<Buf, Fixups, L> {
        self.encoder.emit_byte(0x58 + Rsp::ENC);
        self.encoder
    }
}


pub struct Mov<'a, Buf, Fixups, const L: usize> 
where
    Buf: BufferStorage,
    Fixups: FixupStorage,
{
    encoder: &'a mut Encoder<Buf, Fixups, L>,
}

impl<'a, Buf, Fixups, const L: usize> Mov<'a, Buf, Fixups, L>
where
    Buf: BufferStorage,
    Fixups: FixupStorage,
{
    pub fn rax(&mut self) -> MovDst<'_, Buf, Fixups, L, Rax> {
        MovDst { encoder: self.encoder, _dst: PhantomData }
    }
    
     pub fn rbx(&mut self) -> MovDst<'_, Buf, Fixups, L, Rbx> {
        MovDst { encoder: self.encoder, _dst: PhantomData }
    }
    
    pub fn rcx(&mut self) -> MovDst<'_, Buf, Fixups, L, Rcx> {
        MovDst { encoder: self.encoder, _dst: PhantomData }
    }
    
    pub fn rdx(&mut self) -> MovDst<'_, Buf, Fixups, L, Rdx> {
        MovDst { encoder: self.encoder, _dst: PhantomData }
    }
    
    pub fn rsi(&mut self) -> MovDst<'_, Buf, Fixups, L, Rsi> {
        MovDst { encoder: self.encoder, _dst: PhantomData }
    }
    
    pub fn rdi(&mut self) -> MovDst<'_, Buf, Fixups, L, Rdi> {
        MovDst { encoder: self.encoder, _dst: PhantomData }
    }
    
    pub fn rbp(&mut self) -> MovDst<'_, Buf, Fixups, L, Rbp> {
        MovDst { encoder: self.encoder, _dst: PhantomData }
    }
    
    pub fn rsp(&mut self) -> MovDst<'_, Buf, Fixups, L, Rsp> {
        MovDst { encoder: self.encoder, _dst: PhantomData }
    }
}

pub struct MovDst<'a, Buf, Fixups, const L: usize, D: Reg> 
where
    Buf: BufferStorage,
    Fixups: FixupStorage,
{
    encoder: &'a mut Encoder<Buf, Fixups, L>,
    _dst: PhantomData<D>,
}

impl<'a, Buf, Fixups, const L: usize, D: Reg> MovDst<'a, Buf, Fixups, L, D>
where
    Buf: BufferStorage,
    Fixups: FixupStorage,
{
    fn emit_mov_rr<S: Reg>(&mut self) -> EncoderResult<()> {
        let rex = 0x40 | D::REX | S::REX;
        if rex != 0x40 {
            self.encoder.emit_byte(rex)?;
        }
        self.encoder.emit_byte(0x89)?;
        let modrm = 0xC0 | (D::ENC << 3) | S::ENC;
        self.encoder.emit_byte(modrm)?;
        Ok(())
    }
    
    pub fn rbx(&mut self) -> EncoderResult<&mut Encoder<Buf, Fixups, L>> {
        self.emit_mov_rr::<Rbx>()?;
        Ok(self.encoder)
    }
    
    pub fn rcx(&mut self) -> EncoderResult<&mut Encoder<Buf, Fixups, L>> {
        self.emit_mov_rr::<Rcx>()?;
        Ok(self.encoder)
    }
    
    pub fn rdx(&mut self) -> EncoderResult<&mut Encoder<Buf, Fixups, L>> {
        self.emit_mov_rr::<Rdx>()?;
        Ok(self.encoder)
    }
    
    pub fn rsi(&mut self) -> EncoderResult<&mut Encoder<Buf, Fixups, L>> {
        self.emit_mov_rr::<Rsi>()?;
        Ok(self.encoder)
    }
    
    pub fn rdi(&mut self) -> EncoderResult<&mut Encoder<Buf, Fixups, L>> {
        self.emit_mov_rr::<Rdi>()?;
        Ok(self.encoder)
    }
    
    pub fn rbp(&mut self) -> EncoderResult<&mut Encoder<Buf, Fixups, L>> {
        self.emit_mov_rr::<Rbp>()?;
        Ok(self.encoder)
    }
    
    pub fn rsp(&mut self) -> EncoderResult<&mut Encoder<Buf, Fixups, L>> {
        self.emit_mov_rr::<Rsp>()?;
        Ok(self.encoder)
    }
    
    pub fn imm64(&mut self, val: u64) -> EncoderResult<&mut Encoder<Buf, Fixups, L>> {
        let rex = 0x48 | D::REX;
        if rex != 0x48 {
            self.encoder.emit_byte(rex)?;
        }
        self.encoder.emit_byte(0xB8 + D::ENC)?;
        self.encoder.emit(&val.to_le_bytes())?;
        Ok(self.encoder)
    }
    
    pub fn imm32(&mut self, val: u32) -> EncoderResult<&mut Encoder<Buf, Fixups, L>> {
        let rex = 0x40 | D::REX;
        if rex != 0x40 {
            self.encoder.emit_byte(rex)?;
        }
        self.encoder.emit_byte(0xB8 + D::ENC)?;
        self.encoder.emit(&val.to_le_bytes())?;
        Ok(self.encoder)
    }
}

impl<Buf, Fixups, const LABEL_CAP: usize> Encoder<Buf, Fixups, LABEL_CAP>
where 
    Buf: BufferStorage,
    Fixups: FixupStorage,
{
    pub fn emit(&mut self, bytes: &[u8]) -> EncoderResult<()> {
        self.buffer.extend(bytes)
    }

    pub fn emit_byte(&mut self, byte: u8) -> EncoderResult<()> {
        self.buffer.push(byte)
    }
    
    pub fn bytes(&self) -> &[u8] {
        self.buffer.as_slice()
    }
    
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
    
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }
    
    pub fn remaining(&self) -> usize {
        self.capacity() - self.len()
    }
    
    pub fn lock(&mut self) -> &mut Self {
        let _ = self.prefixes.push(0xF0);
        self
    }
    
    pub fn rep(&mut self) -> &mut Self {
        let _ = self.prefixes.push(0xF3);
        self
    }
    
    pub fn repne(&mut self) -> &mut Self {
        let _ = self.prefixes.push(0xF2);
        self
    }
    
    fn emit_with_prefixes(&mut self, opcode: &[u8], modrm: Option<u8>) {
        let prefixes = core::mem::take(&mut self.prefixes);

        for prefix in prefixes.iter() {
            self.emit_byte(*prefix);
        }

        self.prefixes.clear();
        self.emit(opcode);
        
        if let Some(modrm) = modrm {
            self.emit_byte(modrm);
        }
    }
    
    pub fn label(&mut self, name: &str) -> &mut Self {
        let id = self.next_label_id;
        self.next_label_id = LabelId(id.0 + 1);
        let _ = self.labels.insert(id, self.buffer.len());
        self
    }
    
    pub fn label_id(&mut self) -> LabelId {
        let id = self.next_label_id;
        self.next_label_id = LabelId(id.0 + 1);
        let _ = self.labels.insert(id, self.buffer.len());
        id
    }
    
    pub fn label_at(&mut self, id: LabelId) -> &mut Self {
        let _ = self.labels.insert(id, self.buffer.len());
        self
    }
    
    pub fn add_fixup(&mut self, pos: usize, label: LabelId, kind: FixupKind) {
        self.fixups.push(Fixup { pos, label, kind });
    }
    
    pub fn finalize(&mut self) -> EncoderResult<()> {
        for fixup in self.fixups.iter() {
            let target = self.labels.get(&fixup.label)
                .ok_or_else(|| EncoderError::UndefinedLabel(fixup.label))?;
            
            let offset = *target as isize - (fixup.pos as isize + 4);
            
            match fixup.kind {
                FixupKind::Rel8 => {
                    if offset < -128 || offset > 127 {
                        return Err(EncoderError::Rel8OutOfRange {
                            pos: fixup.pos,
                            offset,
                        });
                    }
                    let bytes = self.buffer.as_mut_slice();
                    bytes[fixup.pos] = (offset as i8) as u8;
                }
                FixupKind::Rel32 => {
                    let offset32 = offset as i32;
                    let bytes = self.buffer.as_mut_slice();
                    bytes[fixup.pos..fixup.pos+4].copy_from_slice(&offset32.to_le_bytes());
                }
                FixupKind::Abs32 => {
                    let val = *target as u32;
                    let bytes = self.buffer.as_mut_slice();
                    bytes[fixup.pos..fixup.pos+4].copy_from_slice(&val.to_le_bytes());
                }
                FixupKind::Abs64 => {
                    let val = *target as u64;
                    let bytes = self.buffer.as_mut_slice();
                    bytes[fixup.pos..fixup.pos+8].copy_from_slice(&val.to_le_bytes());
                }
            }
        }
        Ok(())
    }
        
    pub fn jmp(&mut self, label: LabelId) -> &mut Self {
        self.emit(&[0xE9]);
        let pos = self.buffer.len();
        self.emit(&[0; 4]);
        self.add_fixup(pos, label, FixupKind::Rel32);
        self
    }
    
    pub fn je(&mut self, label: LabelId) -> &mut Self {
        self.emit(&[0x0F, 0x84]);
        let pos = self.buffer.len();
        self.emit(&[0; 4]);
        self.add_fixup(pos, label, FixupKind::Rel32);
        self
    }
    
    pub fn jne(&mut self, label: LabelId) -> &mut Self {
        self.emit(&[0x0F, 0x85]);
        let pos = self.buffer.len();
        self.emit(&[0; 4]);
        self.add_fixup(pos, label, FixupKind::Rel32);
        self
    }
    
    pub fn jmp_short(&mut self, label: LabelId) -> &mut Self {
        self.emit(&[0xEB]);
        let pos = self.buffer.len();
        self.emit(&[0]);
        self.add_fixup(pos, label, FixupKind::Rel8);
        self
    }

    pub fn ret(&mut self) -> EncoderResult<&mut Self> {
        self.emit_byte(0xC3)?;
        Ok(self)
    }

    pub fn push(&mut self) -> Push<Buf, Fixups, LABEL_CAP> {
        Push { encoder: self }
    }
    
    pub fn pop(&mut self) -> Pop<Buf, Fixups, LABEL_CAP> {
        Pop { encoder: self }
    }
    
    pub fn mov(&mut self) -> Mov<Buf, Fixups, LABEL_CAP> {
        Mov { encoder: self }
    }
}

pub struct Encoder<Buf, Fixups, const LABEL_CAP: usize = 32> 
where 
    Buf: BufferStorage,
    Fixups: FixupStorage,
{
    buffer: Buf,
    fixups: Fixups,
    labels: FnvIndexMap<LabelId, usize, LABEL_CAP>,
    next_label_id: LabelId,
    prefixes: Vec<u8, 8>,
    _marker: PhantomData<Buf>,
}

impl<Buf, Fixups, const LABEL_CAP: usize> Encoder<Buf, Fixups, LABEL_CAP>
where 
    Buf: BufferStorage + Default,
    Fixups: FixupStorage + Default,
{
    pub fn new() -> Self {
        Self {
            buffer: Buf::default(),
            fixups: Fixups::default(),
            labels: FnvIndexMap::new(),
            next_label_id: LabelId(0),
            prefixes: Vec::new(),
            _marker: PhantomData,
        }
    }
}

pub type EncoderFixed<const B: usize, const F: usize, const L: usize = 32> = 
    Encoder<Vec<u8, B>, Vec<Fixup, F>, L>;

pub type Encoder1K = EncoderFixed<1024, 64>;
pub type Encoder4K = EncoderFixed<4096, 256>;
pub type Encoder16K = EncoderFixed<16384, 1024>;

pub fn test_fixed() -> Result<Encoder1K, EncoderError> {
    let mut encoder = Encoder1K::new();
    
    let start = encoder.label_id();
    let skip = encoder.label_id();
    
    encoder.push().rax();
    encoder.push().rbx();
    encoder.mov().rbx().rcx();
    encoder.mov().rax().imm64(42);
    encoder.jmp(skip);
    
    encoder.label_at(start); // Mark start label
    encoder.mov().rax().imm64(0);
    
    encoder.label_at(skip);
    encoder.pop().rbx();
    encoder.pop().rax();
    encoder.ret();
    
    encoder.finalize()?;
    
    Ok(encoder)
}