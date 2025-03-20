use std::{fs, path::{Path, PathBuf}};

use abi_stable::{library::lib_header_from_path, reexports::SelfOps};
use shared::{ServiceRoot_Prefix, ServiceRoot_Ref};

fn main() {
   
    let library_path = "./target/debug/library.dll"
        .as_ref_::<Path>()
        .into_::<PathBuf>();
    let absolute_library_path = fs::canonicalize(&library_path).unwrap();
    println!("{:?}", absolute_library_path);
    println!("lib_header_from_path");
    let header = lib_header_from_path(&absolute_library_path).unwrap();
    println!("init_root_module");
    let service_root = header.init_root_module::<ServiceRoot_Ref>().unwrap();

    let mut service = service_root.new()().unwrap();

    service.start();
}