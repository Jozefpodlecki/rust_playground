use std::ptr::null_mut;

use minidump::*;
use windows::{core::PCWSTR, Win32::{Foundation::HANDLE, Storage::FileSystem::{CreateFileW, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_WRITE}, System::Diagnostics::Debug::{MiniDumpWriteDump, MINIDUMP_TYPE}}};

fn to_pcwstr(s: &str) -> PCWSTR {
    let wide: Vec<u16> = OsStr::new(s).encode_wide().chain(once(0)).collect();
    PCWSTR(wide.as_ptr())
}

fn main() -> Result<(), Error> {

    unsafe  {
        let process_handle = 0;
        let process_id = 0;
        let dump_type = MINIDUMP_TYPE(0);

        let file = unsafe {
            CreateFileW(
                to_pcwstr("test.dmp"),
                FILE_GENERIC_WRITE,
                0,
                null_mut(),
                CREATE_ALWAYS,
                FILE_ATTRIBUTE_NORMAL,
                std::ptr::null_mut(),
            )
        }?;

        MiniDumpWriteDump(
            process_handle,
            process_id,
            file_handle,
            dump_type,
            None,
            None,
            None
        )?;

    }
    
    let mut dump = minidump::Minidump::read_path("../testdata/test.dmp")?;
    let system_info = dump.get_stream::<MinidumpSystemInfo>()?;
    let exception = dump.get_stream::<MinidumpException>()?;

    Ok(())
}
