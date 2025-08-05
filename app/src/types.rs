use std::{env, path::PathBuf};
use anyhow::*;

#[derive(Debug, Clone)]
pub struct RunArgs {
    pub cipher_key: Vec<u8>,
    pub aes_xor_key: Vec<u8>,
    pub lpk_dir: String,
    pub output_path: PathBuf,
    pub exe_path: String,
    pub exe_args: Vec<String>,
    pub addr_offset: Option<u64>
}

impl RunArgs {
    pub fn new() -> Result<Self> {

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

        let addr_offset = std::env::var("ADDR_OFFSET")?;
        let addr_offset  = (!addr_offset.is_empty()).then(|| {
            let addr_offset = addr_offset.trim_start_matches("0x");
            u64::from_str_radix(addr_offset, 16).ok()
        }).flatten();

        Ok(Self {
            cipher_key,
            aes_xor_key,
            lpk_dir,
            output_path,
            exe_path,
            exe_args,
            addr_offset
        })
    }
}