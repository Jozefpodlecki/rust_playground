use std::{collections::HashMap, io::{Read, Seek, SeekFrom, Write}, path::PathBuf};
use anyhow::*;
use byteorder::{LittleEndian, ReadBytesExt};
use chrono::DateTime;

use crate::process_dumper::utils::*;

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
pub struct ProcessDump {
    pub win_version: String,
    pub modules: HashMap<String, ProcessModule>,
    pub exports: HashMap<String, Vec<ProcessModuleExport>>,
    pub blocks: Vec<SerializedMemoryBlock>,
}

#[derive(Debug)]
pub struct SerializedMemoryBlock {
    pub data_offset: u64,
    pub block: MemoryBlock,
}

impl ProcessDump {
    pub fn decode<R: Read + Seek>(mut reader: &mut R) -> Result<Self> {
        let win_version = read_string(reader)?;

        let module_count = reader.read_u32::<LittleEndian>()?;
        let mut modules = HashMap::with_capacity(module_count as usize);
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

        let block_count = reader.read_u32::<LittleEndian>()?;
        let mut blocks = Vec::with_capacity(block_count as usize);
        for _ in 0..block_count {
            blocks.push(SerializedMemoryBlock::decode(reader)?);
        }

        Ok(Self {
            win_version,
            modules,
            exports,
            blocks,
        })
    }

   
}

impl SerializedMemoryBlock {
    pub fn decode<R: Read + Seek>(mut reader: &mut R) -> Result<Self> {
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
        let data_len = reader.read_u32::<LittleEndian>()? as usize;
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

    pub fn encode<W: Write + Seek>(&self, writer: &mut W) -> Result<()> {
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

        Ok(())
    }
}