use std::{fs, path::{Path, PathBuf}};
use anyhow::*;
use abi_stable::{library::lib_header_from_path, reexports::SelfOps, std_types::RResult::ROk};
use shared::{ServiceRoot_Prefix, ServiceRoot_Ref};

#[tokio::main]
async fn main() -> Result<()> {
   
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

    let rx = service.start().unwrap();

    loop {
        match rx.recv() {
            std::result::Result::Ok(value) => {

                if value > 5 {
                    println!("stopping");
                    service.stop();
                    break;
                }

            },
            Err(err) => {
                println!("{}", err)
            },
        }
    }

    Ok(())
}