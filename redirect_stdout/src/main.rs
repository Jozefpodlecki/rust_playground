use std::{fs::File, os::windows::io::{FromRawHandle, RawHandle}, ptr};

use anyhow::Result;
use simple_logger::SimpleLogger;
use windows::{core::PCSTR, Win32::{Foundation::HANDLE, Storage::FileSystem::{CreateFileA, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_WRITE, FILE_SHARE_MODE, OPEN_ALWAYS}, System::Console::{SetStdHandle, STD_OUTPUT_HANDLE}}};

#[tokio::main]
async fn main() -> Result<()> {
    
    SimpleLogger::new().env().init().unwrap();

    let filename = b"log.txt\0"; // Null-terminated byte string
    let pcstr = PCSTR::from_raw(filename.as_ptr());

    let file = unsafe { CreateFileA(
        pcstr,
        FILE_GENERIC_WRITE.0,
        FILE_SHARE_MODE(0),
        None,
        OPEN_ALWAYS,
        FILE_ATTRIBUTE_NORMAL,
        Some(HANDLE(std::ptr::null_mut())),
    ) };


    let handle = file.unwrap();

    if handle.is_invalid() {
        panic!("Failed to open log file.");
    }

    let result = unsafe { SetStdHandle(STD_OUTPUT_HANDLE, handle) };

    if result.is_err() {
        panic!("Failed to redirect stdout.");
    }

    // let file = unsafe { File::from_raw_handle(handle as RawHandle) };

    // let fd = file.as_raw_fd();
    // unsafe {
    //     libc::dup2(fd, libc::STDOUT_FILENO);
    // }

    // // Write something to check if it works
    // println!("This will be logged in log.txt");

    /*
    HANDLE new_stdout = CreateFileA("log.txt", ...);
    SetStdHandle(STD_OUTPUT_HANDLE, new_stdout);
    int fd = _open_osfhandle(new_stdout, O_WRONLY|O_TEXT);
    dup2(fd, STDOUT_FILENO);
    close(fd);
    */
    
    Ok(())
}