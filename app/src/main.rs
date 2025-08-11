#![allow(warnings)]

use anyhow::*;

use flexi_logger::Logger;
use std::{ffi::OsStr, fs::File, os::windows::ffi::OsStrExt, process::exit, ptr::null_mut, result::Result::Ok, thread::sleep, time::Duration};
use log::*;
use windows::{core::{PCWSTR, PWSTR}, Win32::{Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY}, System::Threading::{GetCurrentProcess, OpenProcessToken}, UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL}}};
use crate::{processor::*, types::AppConfig};

mod types;
mod process;
mod processor;
mod lpk;
mod sql_migrator;
mod loa_extractor;
mod enum_extractor;
mod utils;
mod models;
mod disassembler;


fn to_pcwstr(s: &str) -> PCWSTR {
    let wide: Vec<u16> = OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    PCWSTR(wide.as_ptr() as *const u16)
}

fn to_pcwstr_owned(s: &str) -> (Vec<u16>, PCWSTR) {
    let wide: Vec<u16> = OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let ptr = PCWSTR(wide.as_ptr());
    (wide, ptr)
}


fn is_elevated() -> Result<bool> {
    unsafe {
        let mut token = Default::default();
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token)?;

        let mut elevation = TOKEN_ELEVATION::default();
        let mut return_len = std::mem::size_of::<TOKEN_ELEVATION>() as u32;

        GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as _),
            return_len,
            &mut return_len,
        )?;

        Ok(elevation.TokenIsElevated != 0)
    }
}

fn main() -> Result<()> {
    let args = AppConfig::new()?;
    // let current_exe = std::env::current_exe()?;
    // let current_exe = current_exe.to_str().unwrap();
    // // let exe_path = to_pcwstr(r"C:\repos\rust_playground\app\target\debug\app.exe");
    // let exe_path = to_pcwstr(r"app.exe");
    // println!("{} {:?}", current_exe, exe_path);
    // let verb = to_pcwstr("runas");
    // let directory = to_pcwstr(r"C:\repos\rust_playground\app\target\debug");
    // let parameters = to_pcwstr("");

    // let result = unsafe {
    //     ShellExecuteW(
    //         None,
    //         verb,
    //         exe_path,
    //         parameters,
    //         directory,
    //         SW_SHOWNORMAL,
    //     )
    // };

    // if result.0 as usize <= 32 {
    //     eprintln!("ShellExecuteW failed with code: {:?}", result.0);
    //     // You may want to exit or handle specific errors here.
    // } else {
    //     println!("Process launched with elevation. {:?}", result.0);
    // }

    // if is_elevated()? {
    //     let test = File::create("C:\\test.txt")?;
    //     sleep(Duration::from_secs(60));
    // }
    // else {
    //     println!("Not elevated.");
    //     sleep(Duration::from_secs(60));
    //     exit(0);
    // }

    Logger::try_with_str(&args.log_level)?.start()?;

    let mut processor = Processor::new();

    if args.cleanup.is_enabled {
        processor.add_step(Box::new(CleanupDirectoryStep::new(
            args.output_path.clone(),
            args.cleanup.files,
            args.cleanup.folders
        )));
    }
    processor.add_step(Box::new(CopyFileStep::new(
        args.game_path.clone(),
        args.output_path.join("lpk"),
        "lpk",
        false)));
    processor.add_step(Box::new(CopyFileStep::new(
        args.game_path.clone(),
        args.output_path.join("upk"),
        "upk",
        true)));
    processor.add_step(Box::new(CopyFileStep::new(
        args.game_path.clone(),
        args.output_path.join("ipk"),
        "ipk",
        true)));
    processor.add_step(Box::new(ExtractLpkStep::new(
        args.cipher_key.clone(),
        args.aes_xor_key.clone(),
        args.output_path.join("lpk"),
        args.output_path.join("lpk")    
    )));
    processor.add_step(Box::new(DecryptUpkStep::new(
        args.output_path.join("upk"),
    )));

    for exe_info in args.exe_paths.clone() {
        if args.process_dumper.is_enabled {
            processor.add_step(Box::new(DumpProcessStep::new(
                exe_info.path.clone(),
                args.output_path.clone(),
                exe_info.args,
                exe_info.launch_method,
            )));
        }

        if args.disassembler.is_enabled {
            processor.add_step(Box::new(DisassembleProcessStep::new(
                args.disassembler.clone(),
                exe_info.path.clone(),
                args.output_path.clone()
            )));
        }
        
        processor.add_step(Box::new(ParseDumpStep::new(
            exe_info.path,
            args.output_path.clone()
        )));
    }

    // processor.add_step(Box::new(CombineDbStep::new(args)));

    if let Err(err) = processor.run() {
        error!("An error occurred whilst processing: {err:?}");
    }
    
    Ok(())
}
