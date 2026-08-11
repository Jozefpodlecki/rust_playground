

use core::{hash::BuildHasherDefault, marker::PhantomData};

use heapless::{IndexMap, String, Vec, index_map::FnvIndexMap};

use crate::encoder::{instructions::*, registers::*, types::*};


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

    pub fn push(&mut self) -> Push<'_, Buf, Fixups, LABEL_CAP> {
        Push::new(self)
    }
    
    pub fn pop(&mut self) -> Pop<'_, Buf, Fixups, LABEL_CAP> {
        Pop::new(self)
    }
    
    pub fn mov(&mut self) -> Mov<'_, Buf, Fixups, LABEL_CAP> {
        Mov::new(self)
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

// pub fn test_fixed() -> Result<Encoder1K, EncoderError> {
//     let mut encoder = Encoder1K::new();
    
//     let start = encoder.label_id();
//     let skip = encoder.label_id();
    
//     encoder.push().rax();
//     encoder.push().rbx();
//     encoder.mov().rbx().rcx();
//     encoder.mov().rax().imm64(42);
//     encoder.jmp(skip);
    
//     encoder.label_at(start); // Mark start label
//     encoder.mov().rax().imm64(0);
    
//     encoder.label_at(skip);
//     encoder.pop().rbx();
//     encoder.pop().rax();
//     encoder.ret();
    
//     encoder.finalize()?;
    
//     Ok(encoder)
// }