#![allow(warnings)]

use std::{env, fs::File, io::Write, path::Path};
use anyhow::*;
use chrono::Local;
use flexi_logger::Logger;
use rusqlite::{Connection, OptionalExtension};
use std::result::Result::Ok;
use log::*;

use crate::{lpk::{get_lpks_dict, LpkInfo}, process_dumper::dump_process, processor::*, sql_migrator::{DbMerger, MergeDirectoryFilter}, types::RunArgs};

mod types;
mod process_dumper;
mod processor;
mod lpk;
mod sql_migrator;
mod loa_extractor;

fn main() -> Result<()> {
    Logger::try_with_str("info")?.start()?;
    dotenvy::dotenv()?;
    
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
    let timestamp = Local::now().format("%Y%m%d_%H%M%S_%3f").to_string();
    let filename = format!("output_{}.duckdb", timestamp);
    info!("Created {}", &filename);
    let duckdb_path = args.output_path.join(filename);
    // extract_lpk(args)?;

    let filter = MergeDirectoryFilter::None;
    // let filter = MergeDirectoryFilter::Include(vec!["SkillEffect"]);
    let merger = DbMerger::new(&duckdb_path, 1000)?;
    merger.setup();
    merger.merge_directory(&sqlite_dir, "data", &filter)?;
    merger.merge_directory(&jss_sqlite_dir, "jss", &filter)?;
    // merger.post_work();
    
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
