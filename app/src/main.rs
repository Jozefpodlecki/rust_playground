#![allow(warnings)]

use std::{env, fs::{self, File}, io::{BufWriter, Write}, path::{Path, PathBuf}};
use anyhow::*;

use chrono::Local;
use flexi_logger::Logger;
use rusqlite::{Connection, OptionalExtension};
use std::result::Result::Ok;
use log::*;

use crate::{enum_extractor::*, lpk::{get_lpks_dict, LpkInfo}, process_dumper::*, processor::*, sql_migrator::{collect_db_file_entries, DbMerger, DuckDb, TransformationStrategy}, types::RunArgs, utils::{save_pretty_hex_dump, save_pretty_hex_dump_from_slice}};

mod types;
mod process_dumper;
mod processor;
mod lpk;
mod sql_migrator;
mod loa_extractor;
mod enum_extractor;
mod utils;

fn main() -> Result<()> {
    Logger::try_with_str("info")?.start()?;
    dotenvy::dotenv()?;

    let args = RunArgs::new()?;
    let sqlite_dir = args.output_path.join("../debug").join(r"data2\EFGame_Extra\ClientData\TableData");
    let jss_sqlite_dir = args.output_path.join("../debug").join(r"data2\EFGame_Extra\ClientData\TableData\jss");
    let enum_path = args.output_path.join(r"data1\Common\StringData\EFGameMsg_Enums.xml");
    
    {
        let file_name = "output_20250805_172138_071.duckdb";
        let duckdb_path = args.output_path.join(file_name);

        let duck_db = DuckDb::new(&duckdb_path)?;

        let script_path = args.output_path.join("refactor.sql");;
        duck_db.prepare_post_work_script(&script_path);
    }

    {
        // let timestamp = Local::now().format("%Y%m%d_%H%M%S_%3f").to_string();
        // let file_name = format!("output_{}.duckdb", timestamp);
        // let duckdb_path = args.output_path.join(file_name);

        // let merger = DbMerger::new(&duckdb_path, 1000)?;
        // merger.setup();
        
        // let enums = extract_enum_maps_from_xml(&enum_path)?;

        // merger.create_enums(enums, "enum")?;
        // // merger.insert_loa_data(&args.output_path)?;

        // let mut entries = collect_db_file_entries(&sqlite_dir)?;

        // let entry = entries.get_mut("EFTable_LanguageGrams").unwrap();
        // entry.strategy = TransformationStrategy::BatchInsert;
        
        // merger.merge(entries, "data")?;

        // let mut entries = collect_db_file_entries(&jss_sqlite_dir)?;

        // merger.merge(entries, "jss")?;
    }
    
    Ok(())
}
