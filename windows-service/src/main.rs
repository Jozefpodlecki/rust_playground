#[macro_use]
extern crate windows_service;

use std::{ffi::OsString, fs::File, io::Write, path::PathBuf};
use windows_service::service_dispatcher;

define_windows_service!(ffi_service_main, service_main);

fn service_main(arguments: Vec<OsString>) {
    let base_path = PathBuf::from(r"C:\repos\rust_playground\windows-service\target\debug");
    let mut file = File::create(base_path.join("from-service.txt")).unwrap();
    
    file.write_all(b"test").unwrap();
    file.write_all(format!("args: {:?}\n", arguments).as_bytes()).unwrap();
}

fn main() -> Result<(), windows_service::Error> {
    service_dispatcher::start("CustomService", ffi_service_main)?;
    Ok(())
}