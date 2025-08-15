#![allow(warnings)]

use anyhow::*;

use app_lib::{config::AppConfig, processor::*};
use flexi_logger::Logger;
use log::*;

fn main() -> Result<()> {
    let args = AppConfig::new()?;

    Logger::try_with_str(&args.log_level)?.start()?;

    let mut processor = Processor::new();

    if args.cleanup.is_enabled {
        processor.add_step(Box::new(CleanupDirectoryStep::new(
            args.output_path.clone(),
            args.cleanup.files.clone(),
            args.cleanup.folders.clone()
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

    for exe_info in args.process_dumper.exe_paths.clone() {
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
            args.disassembler.clone(),
            exe_info.path,
            args.output_path.clone()
        )));
    }

    processor.add_step(Box::new(ExtractIconsStep::new(
        args.output_path.join("upk"),
        args.output_path.clone())));
    processor.add_step(Box::new(CombineDbStep::new(args)));

    if let Err(err) = processor.run() {
        error!("An error occurred whilst processing: {err:?}");
    }
    
    Ok(())
}
