#[macro_use]
extern crate windows_service;

use std::ffi::OsString;
use windows_service::service_dispatcher;

define_windows_service!(ffi_service_main, my_service_main);

fn my_service_main(arguments: Vec<OsString>) {
    println!("test");
}

fn main() -> Result<(), windows_service::Error> {
    service_dispatcher::start("myservice", ffi_service_main)?;
    Ok(())
}