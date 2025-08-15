use std::{collections::HashMap, fs::File, io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write}, ops::Deref, path::{Path, PathBuf}};
use anyhow::*;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use log::*;
use serde::Serialize;
use windows::Win32::System::Diagnostics::Debug::CONTEXT;

use crate::process::{memory::MemoryRegionIterator, snapshot, thread_context::ThreadContext, utils::*, ProcessSnapshot};

#[derive(Serialize)]
pub struct DumpSummary {
    pub entry_point: String,
    pub regions: Vec<MemoryRegionSummary>,
    pub modules: HashMap<String, ModuleSummary>
}

#[derive(Serialize)]
pub struct ModuleSummary {
    pub order: u16,
    pub file_name: String,
    pub entry_point: String,
    pub size: u32,
    pub start_addr: String,
    pub end_addr: String,
}

#[derive(Serialize)]
pub struct MemoryRegionSummary {
    pub module_name: Option<String>,
    pub start_address: String,
    pub end_address: String,
    pub size: u64,
    pub is_executable: bool,
    pub is_readable: bool,
}

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
    file: File,
    pub data_offset: u64,
    block: MemoryBlock,
}

impl Deref for SerializedMemoryBlock {
    type Target = MemoryBlock;

    fn deref(&self) -> &Self::Target {
        &self.block
    }
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
    pub address_rva: u64,
    pub address: u64
}

impl ProcessDump {
    pub fn get_path(exe_path: &Path, output_path: &Path) -> PathBuf {
        let file_name = exe_path.file_stem().unwrap().to_string_lossy();
        let output_bin_path = format!("{file_name}.bin");
        let output_bin_path = output_path.join(output_bin_path);

        output_bin_path
    }

    pub fn save(snapshot: ProcessSnapshot, output_bin_path: &Path) -> Result<Self> {
        let mut file = File::create(output_bin_path)?;
        let cloned = file.try_clone()?;

        let blocks = {
            let mut writer = BufWriter::new(&mut file);

            write_string(&mut writer, &snapshot.win_version)?;
            write_threads(&mut writer, &snapshot.threads)?;
            write_modules(&mut writer, &snapshot.modules)?;
            write_module_exports(&mut writer, &snapshot.exports)?;
        
            info!("Extracting memory regions");
            let modules_vec = snapshot.modules.values().cloned().collect::<Vec<_>>();
            let blocks = write_memory_blocks(cloned, &mut writer, snapshot.blocks, &modules_vec)?;
            blocks
        };

        Ok(Self {
            blocks,
            exports: snapshot.exports,
            modules: snapshot.modules,
            threads: snapshot.threads,
            win_version: snapshot.win_version,
        })
    }

    pub fn open<P: AsRef<Path>>(dump_path: P) -> Result<Self> {
        let mut file = File::open(dump_path)?;
        let cloned = file.try_clone()?;
        let mut reader = BufReader::new(&mut file);

        let win_version = read_string(&mut reader)?;
        let threads = read_threads(&mut reader)?;
        let modules = read_modules(&mut reader)?;
        let exports = read_module_exports(&mut reader)?;
        let blocks = read_memory_blocks(cloned, &mut reader)?;

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
    pub fn read_data(&self) -> Result<impl Read> {
        let mut file = self.file.try_clone()?;
        file.seek(SeekFrom::Start(self.data_offset))?;
        
        let data_len = file.read_u64::<LittleEndian>()?;

        let reader = BufReader::new(file);
        let reader =  reader.take(data_len);
        
        Ok(reader)
    }

    pub fn encode<W: Write + Seek>(&self, writer: &mut W, data: Vec<u8>) -> Result<u64> {
        let block = &self.block;

        writer.write_u64::<LittleEndian>(block.size)?; 
        writer.write_u64::<LittleEndian>(block.base)?; 
        writer.write_u32::<LittleEndian>(block.state)?; 
        writer.write_u32::<LittleEndian>(block.protect)?; 
        writer.write_u8(block.is_readable as u8)?;
        writer.write_u8(block.is_executable as u8)?;
        writer.write_u8(block.module_name.is_some() as u8)?;

        if let Some(module_name) = &block.module_name {
            write_string(writer, module_name)?;
        }

        let data_offset = writer.stream_position()?;
        debug!("Writing block at offset {} with size {}", data_offset, data.len());
        writer.write_all(&(data.len() as u64).to_le_bytes())?;
        writer.write_all(&data)?;

        Ok(data_offset)
    }

    pub fn decode<R: Read + Seek>(file: File, reader: &mut R) -> Result<Self> {
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
            file,
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
    writer.write_u32::<LittleEndian>(modules.len() as u32)?;
    for (name, module) in modules {
        write_string(writer, name)?;
        writer.write_u16::<LittleEndian>(module.order)?;
        writer.write_u8(module.is_dll as u8)?;
        write_string(writer, &module.file_path.to_string_lossy())?;
        write_string(writer, &module.file_name)?;
        writer.write_u64::<LittleEndian>(module.entry_point)?;
        writer.write_u32::<LittleEndian>(module.size)?;
        writer.write_u64::<LittleEndian>(module.base)?;
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
            let address_rva = reader.read_u64::<LittleEndian>()?;
            let address = reader.read_u64::<LittleEndian>()?;
            export_list.push(ProcessModuleExport { 
                name,
                address_rva,
                address
            });
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
            writer.write_u64::<LittleEndian>(export.address_rva)?;
            writer.write_u64::<LittleEndian>(export.address)?;
        }
    }
    Ok(())
}

fn read_memory_blocks<R: Read + Seek>(file: File, reader: &mut R) -> Result<Vec<SerializedMemoryBlock>> {
    let block_count = reader.read_u32::<LittleEndian>()?;
    debug!("Reading blocks: {block_count}");
    let mut blocks: Vec<SerializedMemoryBlock> = Vec::with_capacity(block_count as usize);
    for _ in 0..block_count {
        blocks.push(SerializedMemoryBlock::decode(file.try_clone()?, reader)?);
    }
    
    Ok(blocks)
}

fn write_memory_blocks<W: Write + Seek>(
    file: File,
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
            file: file.try_clone()?,
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