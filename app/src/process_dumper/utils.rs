use std::io::{Read, Write};

use anyhow::*;
use byteorder::{LittleEndian, ReadBytesExt};
use windows::Win32::System::SystemInformation::{GetVersionExW, OSVERSIONINFOW};

use crate::process_dumper::types::ProcessModule;

pub unsafe fn get_windows_version() -> Result<String> {
    let mut info = OSVERSIONINFOW::default();
    info.dwOSVersionInfoSize = std::mem::size_of::<OSVERSIONINFOW>() as u32;

    GetVersionExW(&mut info as *mut _ as *mut _)?;

    Ok(format!("{}.{}.{}", info.dwMajorVersion, info.dwMinorVersion, info.dwBuildNumber))
}

pub fn match_module<'a>(
    base: u64,
    modules: &'a [ProcessModule],
) -> Option<&'a ProcessModule> {
    modules.iter().find(|module| {
        let module_start = module.base as usize;
        let module_end = module_start + module.size as usize;
        let block_base = base as usize;

        block_base >= module_start && block_base < module_end
    })
}

pub fn write_string<W: Write>(writer: &mut W, s: &str) -> Result<()> {
    let bytes = s.as_bytes();
    writer.write_all(&(bytes.len() as u32).to_le_bytes())?;
    writer.write_all(bytes)?;
    Ok(())
}

pub fn read_string<R: Read>(reader: &mut R) -> Result<String> {
    let len = reader.read_u32::<LittleEndian>()? as usize;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;

    String::from_utf8(buf).map_err(|e| anyhow!("Invalid UTF-8 string: {}", e))
}

pub fn read_bool<R: Read>(reader: &mut R) -> Result<bool> {
    let byte = reader.read_u8()?;
    match byte {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(anyhow!("Invalid boolean value: {}", byte)),
    }
}