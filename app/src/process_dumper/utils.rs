use anyhow::*;
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
