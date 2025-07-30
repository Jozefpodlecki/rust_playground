use std::env;
use anyhow::*;
use flexi_logger::Logger;
use hex_literal::hex;

use crate::{processor::{extract_lpk, parse_ue3_object}, types::RunArgs};

mod processor;
mod types;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    Logger::try_with_str("info")?.start()?;
    dotenvy::dotenv()?;
    
    let cipher_key = std::env::var("CIPHER_KEY")?.as_bytes().to_vec();
    let aes_xor_key = std::env::var("AES_XOR_KEY")?.as_bytes().to_vec();
    let lpk_dir = std::env::var("LPK_PATH")?;
    // let output_path = env::current_dir()?.to_str().unwrap().to_string();
    let output_path = env::current_exe().unwrap().parent().unwrap().to_string_lossy().to_string();

    let args = RunArgs {
        cipher_key,
        aes_xor_key,
        lpk_dir,
        output_path
    };

    parse_ue3_object(args);

    // extract_lpk(args)?;
    
    Ok(())
}
