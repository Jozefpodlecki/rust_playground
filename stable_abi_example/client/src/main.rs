use std::{fs, path::{Path, PathBuf}};
use anyhow::*;
use abi_stable::{library::lib_header_from_path, reexports::SelfOps, std_types::RResult::ROk};
use shared::{models::Command, traits::AsBoxedReceiver, ServiceRoot_Prefix, ServiceRoot_Ref};

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

    let mut service = service_root.new_tokio()().unwrap();

   
    let raw_ptr = service.start();
    let mut rx = raw_ptr.as_boxed_receiver::<Command>();
    let mut it = 0;

    loop {
        let command = rx.recv().await.unwrap();
        
        println!("{:?}", command);

        if it > 5 {
            println!("stopping");
            service.stop();
            break;
        }

        it += 1;
    }

    // let mut service = service_root.new()().unwrap();

    // let rx = service.start().unwrap();

    // loop {
    //     match rx.recv() {
    //         std::result::Result::Ok(value) => {

    //             if value > 5 {
    //                 println!("stopping");
    //                 service.stop();
    //                 break;
    //             }

    //         },
    //         Err(err) => {
    //             println!("{}", err)
    //         },
    //     }
    // }

    Ok(())
}