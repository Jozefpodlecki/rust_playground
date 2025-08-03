#![allow(warnings)]

use std::{env, fs::File, io::{BufWriter, Write}, path::Path};
use anyhow::*;
use chrono::Local;
use flexi_logger::Logger;
use rusqlite::{Connection, OptionalExtension};
use std::result::Result::Ok;
use log::*;

use crate::{enum_extractor::*, lpk::{get_lpks_dict, LpkInfo}, process_dumper::{dump_process, get_windows_version}, processor::*, sql_migrator::{collect_db_file_entries, DbMerger, MergeDirectoryFilter, TransformationStrategy}, types::RunArgs};

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
    
    let version = unsafe { get_windows_version()? };
    info!("Windows version: {version}");

    let cipher_key = std::env::var("CIPHER_KEY")?.as_bytes().to_vec();
    let aes_xor_key = std::env::var("AES_XOR_KEY")?;
    let aes_xor_key = hex::decode(aes_xor_key)?;

    let lpk_dir = std::env::var("LPK_PATH")?;
    let exe_path = std::env::var("EXE_PATH")?;
    // let output_path = env::current_dir()?.to_str().unwrap().to_string();
    let output_path = env::current_exe().unwrap().parent().unwrap().to_owned();
    
    let args_str = env::var("EXE_ARGS").unwrap_or_default();
    let mut exe_args: Vec<String> = vec![exe_path.clone()];
    exe_args.extend(args_str.split(',').map(|s| s.to_string()));

    let args = RunArgs {
        cipher_key,
        aes_xor_key,
        lpk_dir,
        output_path,
        exe_path,
        exe_args
    };

    let sqlite_dir = args.output_path.join("../debug").join(r"data2\EFGame_Extra\ClientData\TableData");
    let jss_sqlite_dir = args.output_path.join("../debug").join(r"data2\EFGame_Extra\ClientData\TableData\jss");
    
    // let filename = ".duckdb";

    let timestamp = Local::now().format("%Y%m%d_%H%M%S_%3f").to_string();
    let filename = format!("output_{}.duckdb", timestamp);
    info!("Created {}", &filename);

    let duckdb_path = args.output_path.join(filename);
    // extract_lpk(args)?;

    // let path = Path::new(&args.output_path).join(r"data3\EFGame_Extra\ClientData\XmlData\Prepare\10001.loa");
    // let value = parse_ue3_object(&path)?;
    // println!("{value}");

    // let writer = File::create("output123.json")?;
    // serde_json::to_writer_pretty(writer, &value)?;
    

    let filter = MergeDirectoryFilter::None;
    // let filter = MergeDirectoryFilter::Exclude(vec!["LanguageGrams"]);
    // // let filter = MergeDirectoryFilter::Include(vec!["SkillEffect"]);
   
    let merger = DbMerger::new(&duckdb_path, 1000)?;
    merger.setup();

    let enum_path = args.output_path.join(r"data1\Common\StringData\EFGameMsg_Enums.xml");
    let enums = extract_enum_maps_from_xml(&enum_path)?;

    // merger.create_enums(enums, "enum")?;
    merger.insert_loa_data(&args.output_path)?;

    return Ok(());

    let mut entries = collect_db_file_entries(&sqlite_dir, &filter)?;
    let entry = entries.get_mut("LanguageGrams").unwrap();
    entry.strategy = TransformationStrategy::BatchInsert;
    
    merger.merge(entries, "data")?;
    // merger.merge_directory(&jss_sqlite_dir, "jss", &filter)?;
    merger.post_work()?;
    
    Ok(())
}


    // open_sqlite_db(args)?;

    // match unsafe { dump_process(args) } {
    //     Ok(_) => {
    //         info!("done");
    //     },
    //     Err(err) => {
    //         let backtrace = err.backtrace();
    //         error!("{err}");
    //         // error!("{backtrace}");
    //     },
    // }
    // println!("{}", decrypt("YGI3SWD3SHKA3D1EG9KMHCXZM.upk"));
    // parse_ue3_object_1(args)?;
    // parse_ue3_object_1(args)?;

    // extract_lpk(args)?;
