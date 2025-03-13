use std::{fs::File, io::Write, os::windows::io::{AsHandle, AsRawHandle, FromRawHandle, RawHandle}, ptr};

use anyhow::Result;
use simple_logger::SimpleLogger;
use windows::{core::PCSTR, Win32::{Foundation::HANDLE, Storage::FileSystem::{CreateFileA, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_WRITE, FILE_SHARE_MODE, OPEN_ALWAYS}, System::Console::{SetStdHandle, STD_OUTPUT_HANDLE}}};

#[tokio::main]
async fn main() -> Result<()> {
    
    SimpleLogger::new().env().init().unwrap();

    let file =  File::create("log.txt").unwrap();
    let raw_handle = file.as_raw_handle();
    let handle: HANDLE = HANDLE(raw_handle as *mut _);

    let result = unsafe { SetStdHandle(STD_OUTPUT_HANDLE, handle) };

    if result.is_err() {
        panic!("Failed to redirect stdout.");
    }

    let int_ptr: isize = handle.0 as isize;
    let file_descriptor = unsafe { libc::open_osfhandle(int_ptr, libc::O_WRONLY | libc::O_TEXT) };
    let stdout = 1;
    let new_file_descriptor = unsafe { libc::dup2(file_descriptor, stdout) };
    println!("{}", new_file_descriptor);

    unsafe { libc::close(new_file_descriptor) };

    println!("test");
    
    Ok(())
}