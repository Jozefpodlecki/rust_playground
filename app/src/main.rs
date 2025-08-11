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
mod export_dump;

fn main() -> Result<()> {
    let args = AppConfig::new()?;

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

        processor.add_step(Box::new(ExtractPeStep::new(
            exe_info.path.clone(),
            args.output_path.clone()
        )));
        
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
