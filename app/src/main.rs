use std::env;
use anyhow::*;
use flexi_logger::Logger;
use std::result::Result::Ok;
use log::*;

use crate::{process_dumper::dump_process, processor::{decrypt, extract_lpk, parse_ue3_object_1}, types::RunArgs};

mod process_dumper;
mod processor;
mod types;
mod utils;
mod sql_migrator;

#[tokio::main]
async fn main() -> Result<()> {
    Logger::try_with_str("info")?.start()?;
    dotenvy::dotenv()?;
    
    let cipher_key = std::env::var("CIPHER_KEY")?.as_bytes().to_vec();
    let aes_xor_key = std::env::var("AES_XOR_KEY")?.as_bytes().to_vec();
    let lpk_dir = std::env::var("LPK_PATH")?;
    let exe_path = std::env::var("EXE_PATH")?;
    // let output_path = env::current_dir()?.to_str().unwrap().to_string();
    let output_path = env::current_exe().unwrap().parent().unwrap().to_string_lossy().to_string();

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

    match unsafe { dump_process(args) } {
        Ok(_) => {
            info!("done");
        },
        Err(err) => {
            let backtrace = err.backtrace();
            error!("{err}");
            // error!("{backtrace}");
        },
    }
    // println!("{}", decrypt("YGI3SWD3SHKA3D1EG9KMHCXZM.upk"));
    // parse_ue3_object_1(args)?;
    // parse_ue3_object_1(args)?;

    // extract_lpk(args)?;
    
    Ok(())
}
