use std::{collections::HashMap, io::{Read, Seek, SeekFrom, Write}, path::PathBuf};
use anyhow::*;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use log::*;
use windows::Win32::System::Diagnostics::Debug::CONTEXT;

use crate::process_dumper::{memory::MemoryRegionIterator, utils::*};

#[derive(Debug)]
pub struct ProcessDump {
    pub win_version: String,
    pub modules: HashMap<String, ProcessModule>,
    pub exports: HashMap<String, Vec<ProcessModuleExport>>,
    pub blocks: Vec<SerializedMemoryBlock>,
    pub threads: Vec<ThreadContext>
}

#[derive(Debug, Clone)]
pub struct MemoryBlock {
    pub size: u64,
    pub base: u64,
    pub state: u32,
    pub protect: u32,
    pub module_name: Option<String>,
    pub is_readable: bool,
    pub is_executable: bool,
}

#[derive(Debug)]
pub struct SerializedMemoryBlock {
    pub data_offset: u64,
    pub block: MemoryBlock,
}

#[derive(Debug, Clone)]
pub struct ProcessModule {
    pub order: u16,
    pub is_dll: bool,
    pub file_path: PathBuf,
    pub file_name: String,
    pub entry_point: u64,
    pub size: u32,
    pub base: u64,
}

#[derive(Debug, Clone)]
pub struct ProcessModuleExport {
    pub name: String,
    pub address: u64
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ThreadContext {
    // Control
    pub rip: u64,
    pub rsp: u64,
    pub rbp: u64,
    pub eflags: u32,
    pub mxcsr: u32,

    // General-purpose registers
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,

    // Segment registers (optional, rarely needed)
    pub cs: u16,
    pub ds: u16,
    pub es: u16,
    pub fs: u16,
    pub gs: u16,
    pub ss: u16,

    // Debug registers (optional but useful for hardware breakpoints)
    pub dr0: u64,
    pub dr1: u64,
    pub dr2: u64,
    pub dr3: u64,
    pub dr6: u64,
    pub dr7: u64,
}

impl ThreadContext {
    pub fn new(context: CONTEXT) -> Self {
        Self {
            rip: context.Rip,
            rsp: context.Rsp,
            rbp: context.Rbp,
            eflags: context.EFlags,
            mxcsr: context.MxCsr,
            rax: context.Rax,
            rbx: context.Rbx,
            rcx: context.Rcx,
            rdx: context.Rdx,
            rsi: context.Rsi,
            rdi: context.Rdi,
            r8: context.R8,
            r9: context.R9,
            r10: context.R10,
            r11: context.R11,
            r12: context.R12,
            r13: context.R13,
            r14: context.R14,
            r15: context.R15,
            cs: context.SegCs,
            ds: context.SegDs,
            es: context.SegEs,
            fs: context.SegFs,
            gs: context.SegGs,
            ss: context.SegSs,
            dr0: context.Dr0,
            dr1: context.Dr1,
            dr2: context.Dr2,
            dr3: context.Dr3,
            dr6: context.Dr6,
            dr7: context.Dr7,
        }
    }

    pub fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(Self {
            rip: reader.read_u64::<LittleEndian>()?,
            rsp: reader.read_u64::<LittleEndian>()?,
            rbp: reader.read_u64::<LittleEndian>()?,
            eflags: reader.read_u32::<LittleEndian>()?,
            mxcsr: reader.read_u32::<LittleEndian>()?,
            rax: reader.read_u64::<LittleEndian>()?,
            rbx: reader.read_u64::<LittleEndian>()?,
            rcx: reader.read_u64::<LittleEndian>()?,
            rdx: reader.read_u64::<LittleEndian>()?,
            rsi: reader.read_u64::<LittleEndian>()?,
            rdi: reader.read_u64::<LittleEndian>()?,
            r8: reader.read_u64::<LittleEndian>()?,
            r9: reader.read_u64::<LittleEndian>()?,
            r10: reader.read_u64::<LittleEndian>()?,
            r11: reader.read_u64::<LittleEndian>()?,
            r12: reader.read_u64::<LittleEndian>()?,
            r13: reader.read_u64::<LittleEndian>()?,
            r14: reader.read_u64::<LittleEndian>()?,
            r15: reader.read_u64::<LittleEndian>()?,
            cs: reader.read_u16::<LittleEndian>()?,
            ds: reader.read_u16::<LittleEndian>()?,
            es: reader.read_u16::<LittleEndian>()?,
            fs: reader.read_u16::<LittleEndian>()?,
            gs: reader.read_u16::<LittleEndian>()?,
            ss: reader.read_u16::<LittleEndian>()?,
            dr0: reader.read_u64::<LittleEndian>()?,
            dr1: reader.read_u64::<LittleEndian>()?,
            dr2: reader.read_u64::<LittleEndian>()?,
            dr3: reader.read_u64::<LittleEndian>()?,
            dr6: reader.read_u64::<LittleEndian>()?,
            dr7: reader.read_u64::<LittleEndian>()?,
        })
    }

    pub fn encode<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u64::<LittleEndian>(self.rip)?;
        writer.write_u64::<LittleEndian>(self.rsp)?;
        writer.write_u64::<LittleEndian>(self.rbp)?;
        writer.write_u32::<LittleEndian>(self.eflags)?;
        writer.write_u32::<LittleEndian>(self.mxcsr)?;
        writer.write_u64::<LittleEndian>(self.rax)?;
        writer.write_u64::<LittleEndian>(self.rbx)?;
        writer.write_u64::<LittleEndian>(self.rcx)?;
        writer.write_u64::<LittleEndian>(self.rdx)?;
        writer.write_u64::<LittleEndian>(self.rsi)?;
        writer.write_u64::<LittleEndian>(self.rdi)?;
        writer.write_u64::<LittleEndian>(self.r8)?;
        writer.write_u64::<LittleEndian>(self.r9)?;
        writer.write_u64::<LittleEndian>(self.r10)?;
        writer.write_u64::<LittleEndian>(self.r11)?;
        writer.write_u64::<LittleEndian>(self.r12)?;
        writer.write_u64::<LittleEndian>(self.r13)?;
        writer.write_u64::<LittleEndian>(self.r14)?;
        writer.write_u64::<LittleEndian>(self.r15)?;
        writer.write_u16::<LittleEndian>(self.cs)?;
        writer.write_u16::<LittleEndian>(self.ds)?;
        writer.write_u16::<LittleEndian>(self.es)?;
        writer.write_u16::<LittleEndian>(self.fs)?;
        writer.write_u16::<LittleEndian>(self.gs)?;
        writer.write_u16::<LittleEndian>(self.ss)?;
        writer.write_u64::<LittleEndian>(self.dr0)?;
        writer.write_u64::<LittleEndian>(self.dr1)?;
        writer.write_u64::<LittleEndian>(self.dr2)?;
        writer.write_u64::<LittleEndian>(self.dr3)?;
        writer.write_u64::<LittleEndian>(self.dr6)?;
        writer.write_u64::<LittleEndian>(self.dr7)?;
        Ok(())
    }
}

impl ProcessDump {
    pub fn new_with_encode<W: Write + Seek>(
        writer: &mut W,
        win_version: String,
        threads: Vec<ThreadContext>,
        modules: HashMap<String, ProcessModule>,
        exports: HashMap<String, Vec<ProcessModuleExport>>,
        block_iter: MemoryRegionIterator,
        modules_vec: &[ProcessModule]
    ) -> Result<Self> {
        write_string(writer, &win_version)?;
        write_threads(writer, &threads)?;
        write_modules(writer, &modules)?;
        write_module_exports(writer, &exports)?;
       
        info!("Extracting memory regions");
        let blocks = write_memory_blocks(writer, block_iter, modules_vec)?;

        Ok(Self {
            blocks,
            exports,
            modules,
            threads,
            win_version
        })
    }

    pub fn decode<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        let win_version = read_string(reader)?;
        let threads = read_threads(reader)?;
        let modules = read_modules(reader)?;
        let exports = read_module_exports(reader)?;
        let blocks = read_memory_blocks(reader)?;

        Ok(Self {
            win_version,
            modules,
            exports,
            blocks,
            threads
        })
    }

   
}

impl SerializedMemoryBlock {
    pub fn encode<W: Write + Seek>(&self, writer: &mut W, data: Vec<u8>) -> Result<u64> {
        let block = &self.block;
            
        writer.write_all(&block.size.to_le_bytes())?;
        writer.write_all(&block.base.to_le_bytes())?;
        writer.write_all(&block.state.to_le_bytes())?;
        writer.write_all(&block.protect.to_le_bytes())?;

        writer.write_all(&[block.is_readable as u8])?;
        writer.write_all(&[block.is_executable as u8])?;

        writer.write_all(&[block.module_name.is_some() as u8])?;
        if let Some(module_name) = &block.module_name {
            write_string(writer, module_name)?;
        }

        let data_offset = writer.stream_position()?;
        debug!("Writing block at offset {} with size {}", data_offset, data.len());
        writer.write_all(&(data.len() as u64).to_le_bytes())?;
        writer.write_all(&data)?;

        Ok(data_offset)
    }

    pub fn decode<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        let size = reader.read_u64::<LittleEndian>()?;
        let base = reader.read_u64::<LittleEndian>()?;
        let state = reader.read_u32::<LittleEndian>()?;
        let protect = reader.read_u32::<LittleEndian>()?;

        let is_readable = read_bool(reader)?;
        let is_executable = read_bool(reader)?;

        let has_module = read_bool(reader)?;
        let module_name = if has_module {
            Some(read_string(reader)?)
        } else {
            None
        };

        let data_offset = reader.stream_position()? as u64;
        let data_len = reader.read_u64::<LittleEndian>()?;
        reader.seek(SeekFrom::Current(data_len as i64))?;

        let block = MemoryBlock {
            size,
            base,
            state,
            protect,
            is_readable,
            is_executable,
            module_name,
        };

        Ok(SerializedMemoryBlock {
            data_offset,
            block,
        })
    }
}

fn read_threads<R: Read>(reader: &mut R) -> Result<Vec<ThreadContext>> {
    let thread_count: u32 = reader.read_u32::<LittleEndian>()?;
    let mut threads: Vec<ThreadContext> = Vec::with_capacity(thread_count as usize);
    for _ in 0..thread_count {
        threads.push(ThreadContext::decode(reader)?);
    }

    Ok(threads)
}

fn write_threads<W: Write>(writer: &mut W, threads: &Vec<ThreadContext>) -> Result<()> {
    writer.write_u32::<LittleEndian>(threads.len() as u32)?;
    for thread in threads.iter() {
        thread.encode(writer)?;
    }

    Ok(())
}

fn read_modules<R: Read>(reader: &mut R) -> Result<HashMap<String, ProcessModule>> {
    let module_count = reader.read_u32::<LittleEndian>()?;
    debug!("Reading Modules: {module_count}");
    let mut modules: HashMap<String, ProcessModule> = HashMap::with_capacity(module_count as usize);
    for _ in 0..module_count {
        let name = read_string(reader)?;
        let order = reader.read_u16::<LittleEndian>()?;
        let is_dll = read_bool(reader)?;
        let file_path = PathBuf::from(read_string(reader)?);
        let file_name = read_string(reader)?;
        let entry_point = reader.read_u64::<LittleEndian>()?;
        let size = reader.read_u32::<LittleEndian>()?;
        let base = reader.read_u64::<LittleEndian>()?;

        modules.insert(name.clone(), ProcessModule {
            order,
            is_dll,
            file_path,
            file_name,
            entry_point,
            size,
            base,
        });
    }

    Ok(modules)
}

fn write_modules<W: Write>(writer: &mut W, modules: &HashMap<String, ProcessModule>) -> Result<()> {
    writer.write_all(&(modules.len() as u32).to_le_bytes())?;
    for (name, module) in modules {
        write_string(writer, name)?;
        writer.write_all(&module.order.to_le_bytes())?;
        writer.write_all(&[module.is_dll as u8])?;
        write_string(writer, &module.file_path.to_string_lossy())?;
        write_string(writer, &module.file_name)?;
        writer.write_all(&module.entry_point.to_le_bytes())?;
        writer.write_all(&module.size.to_le_bytes())?;
        writer.write_all(&module.base.to_le_bytes())?;
    }
    Ok(())
}

pub fn read_module_exports<R: Read>(reader: &mut R) -> Result<HashMap<String, Vec<ProcessModuleExport>>> {
    let export_count = reader.read_u32::<LittleEndian>()?;
    let mut exports = HashMap::with_capacity(export_count as usize);

    for _ in 0..export_count {
        let module_name = read_string(reader)?;
        let entry_count = reader.read_u32::<LittleEndian>()?;
        let mut export_list = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            let name = read_string(reader)?;
            let address = reader.read_u64::<LittleEndian>()?;
            export_list.push(ProcessModuleExport { name, address });
        }

        exports.insert(module_name, export_list);
    }

    Ok(exports)
}

fn write_module_exports<W: Write>(writer: &mut W, exports: &HashMap<String, Vec<ProcessModuleExport>>) -> Result<()> {
    writer.write_all(&(exports.len() as u32).to_le_bytes())?;
    for (module_name, export_list) in exports {
        write_string(writer, module_name)?;
        writer.write_all(&(export_list.len() as u32).to_le_bytes())?;

        for export in export_list {
            write_string(writer, &export.name)?;
            writer.write_all(&export.address.to_le_bytes())?;
        }
    }
    Ok(())
}

fn read_memory_blocks<R: Read + Seek>(reader: &mut R) -> Result<Vec<SerializedMemoryBlock>> {
    let block_count = reader.read_u32::<LittleEndian>()?;
    debug!("Reading blocks: {block_count}");
    let mut blocks: Vec<SerializedMemoryBlock> = Vec::with_capacity(block_count as usize);
    for _ in 0..block_count {
        blocks.push(SerializedMemoryBlock::decode(reader)?);
    }
    
    Ok(blocks)
}

fn write_memory_blocks<W: Write + Seek>(
    writer: &mut W,
    block_iter: MemoryRegionIterator,
    modules: &[ProcessModule],
) -> Result<Vec<SerializedMemoryBlock>> {
    let mut blocks = Vec::new();
    let mut count = 0u32;

    let count_pos = writer.stream_position()?;
    writer.write_all(&count.to_le_bytes())?;

    for block in block_iter {
        let (mut block, data) = block?;
        count += 1;

        block.module_name = match_module(block.base, modules).map(|m| m.file_name.clone());

        let mut serialized = SerializedMemoryBlock {
            data_offset: 0,
            block,
        };

        let data_offset = serialized.encode(writer, data)?;
        serialized.data_offset = data_offset;

        blocks.push(serialized);
    }

    writer.seek(SeekFrom::Start(count_pos))?;
    writer.write_all(&count.to_le_bytes())?;

    Ok(blocks)
}