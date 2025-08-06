use std::{io::{Read, Seek, SeekFrom, Write}, path::PathBuf};
use anyhow::*;
use byteorder::{LittleEndian, ReadBytesExt};

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
pub struct MemoryBlock {
    pub size: u64,
    pub base: u64,
    pub state: u32,
    pub protect: u32,
    pub module: Option<ProcessModule>,
    pub is_readable: bool,
    pub is_executable: bool,
}


#[derive(Debug)]
pub struct SerializedMemoryBlock {
    pub data_offset: u64,
    pub block: MemoryBlock,
}

impl SerializedMemoryBlock {
    pub fn decode<R: Read + Seek>(mut reader: &mut R) -> Result<Self> {
        let size = reader.read_u64::<LittleEndian>()?;
        let base = reader.read_u64::<LittleEndian>()?;
        let state = reader.read_u32::<LittleEndian>()?;
        let protect = reader.read_u32::<LittleEndian>()?;

        let is_readable = read_bool(&mut reader)?;
        let is_executable = read_bool(&mut reader)?;

        let has_module = read_bool(&mut reader)?;
        let module = if has_module {
            let order = reader.read_u16::<LittleEndian>()?;
            let is_dll = read_bool(&mut reader)?;

            let file_path_str = read_string(&mut reader)?;
            let file_path = PathBuf::from(file_path_str);

            let file_name = read_string(&mut reader)?;
            let entry_point = reader.read_u64::<LittleEndian>()?;
            let size = reader.read_u32::<LittleEndian>()?;
            let base = reader.read_u64::<LittleEndian>()?;

            Some(ProcessModule {
                order,
                is_dll,
                file_path,
                file_name,
                entry_point,
                size,
                base,
            })
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
            module,
            is_readable,
            is_executable,
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

        writer.write_all(&[block.module.is_some() as u8])?;
        if let Some(module) = &block.module {
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
}