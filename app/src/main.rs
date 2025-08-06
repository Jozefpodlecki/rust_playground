#![allow(warnings)]

use std::{env, fs::{self, File}, io::{BufWriter, Write}, path::{Path, PathBuf}};
use anyhow::*;

use chrono::Local;
use flexi_logger::Logger;
use rusqlite::{Connection, OptionalExtension};
use std::result::Result::Ok;
use log::*;

use crate::{enum_extractor::*, process_dumper::*, processor::*, sql_migrator::*, types::AppConfig};

mod types;
mod process_dumper;
mod processor;
mod lpk;
mod sql_migrator;
mod loa_extractor;
mod enum_extractor;
mod utils;
mod models;

fn main() -> Result<()> {
    let args = AppConfig::new()?;
    Logger::try_with_str(&args.log_level)?.start()?;
    
    let mut processor = Processor::new();

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
        processor.add_step(Box::new(DumpProcessStep::new(
            exe_info.path,
            args.output_path.clone(),
            exe_info.args,
            exe_info.launch_method,
        )));
    }

    processor.add_step(Box::new(CombineDbStep::new(args)));

    processor.run()?;

    {
        // let file_name = "output_20250805_172138_071.duckdb";
        // let duckdb_path = args.output_path.join(file_name);

        // let merger = DbMerger::new(&duckdb_path, 1000)?;
        // merger.insert_loa_data(&args.output_path)?;

        // let mut process_dumper = ProcessDumper::new(&args.exe_path, &args.output_path)?;
        // merger.insert_assembly(process_dumper, &args)?;

        // let duck_db = DuckDb::new(&duckdb_path)?;

        // let script_path = args.output_path.join("refactor.sql");;
        // duck_db.prepare_post_work_script(&script_path);
    }
    
    Ok(())
}
