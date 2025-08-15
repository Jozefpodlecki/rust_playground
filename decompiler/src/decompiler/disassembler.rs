use std::{fs::File, io::{BufReader, Cursor, Read, Seek, SeekFrom, Take}};

use anyhow::Result;

use crate::decompiler::stream::DisasmStream;

pub struct MemorySource<'a>(&'a [u8]);

pub struct FileSource(File);

pub struct Disassembler<S> {
    source: S,
    addr: u64,
    buf_size: usize
}

impl<S> Disassembler<S> {
    fn with_source(source: S, addr: u64, buf_size: usize) -> Result<Self> {
        Ok(Self {
            source,
            addr,
            buf_size
        })
    }
}

impl<'a> Disassembler<MemorySource<'a>> {
    pub fn from_memory(data: &'a [u8], addr: u64, buf_size: usize) -> Result<Self> {
        Self::with_source(MemorySource(data), addr, buf_size)
    }

    pub fn disasm_all(self) -> Result<DisasmStream<Cursor<&'a [u8]>>> {
        let reader = Cursor::new(self.source.0);
        DisasmStream::new(reader, self.addr, self.buf_size)
    }

    pub fn disasm_from_addr(&self, addr: u64) -> Result<DisasmStream<Cursor<&'a [u8]>>> {
        let mut reader = Cursor::new(self.source.0);
        reader.seek(SeekFrom::Start(addr - self.addr))?;
        DisasmStream::new(reader, addr, self.buf_size)
    }

    pub fn disasm_to_addr(&self, addr: u64) -> Result<DisasmStream<Take<Cursor<&'a [u8]>>>> {
        let reader = Cursor::new(self.source.0);
        let reader = reader.take(addr - self.addr);
        DisasmStream::new(reader, addr, self.buf_size)
    }
}

impl Disassembler<FileSource> {
    pub fn from_file(file: File, addr: u64, buf_size: usize) -> Result<Self> {
        Self::with_source(FileSource(file), addr, buf_size)
    }

    pub fn disasm_all(self) -> Result<DisasmStream<BufReader<File>>> {
        let reader = BufReader::new(self.source.0);
        DisasmStream::new(reader, self.addr, self.buf_size)
    }

    pub fn disasm_from_addr(&self, addr: u64) -> Result<DisasmStream<BufReader<File>>> {
        let mut reader = BufReader::new(self.source.0.try_clone()?);
        reader.seek(SeekFrom::Start(addr - self.addr))?;
        DisasmStream::new(reader, addr, self.buf_size)
    }

    pub fn disasm_from_to_addr(&self, start_addr: u64, end_addr: u64) -> Result<DisasmStream<Take<BufReader<File>>>> {
        let mut reader = BufReader::new(self.source.0.try_clone()?);
        reader.seek(SeekFrom::Start(start_addr - self.addr))?;
        let reader = reader.take(end_addr - start_addr);
        DisasmStream::new(reader, start_addr, self.buf_size)
    }

    pub fn disasm_to_addr(&self, addr: u64) -> Result<DisasmStream<Take<BufReader<File>>>> {
        let reader = BufReader::new(self.source.0.try_clone()?);
        let reader = reader.take(addr - self.addr);
        DisasmStream::new(reader, addr, self.buf_size)
    }
}