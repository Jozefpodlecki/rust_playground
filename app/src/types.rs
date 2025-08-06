use std::{env, path::PathBuf, time::Duration};
use anyhow::*;

#[derive(Debug, Clone)]
pub struct RunArgs {
    pub cipher_key: Vec<u8>,
    pub aes_xor_key: Vec<u8>,
    pub game_path: PathBuf,
    pub output_path: PathBuf,
    pub exe_path: PathBuf,
    pub exe_args: Vec<String>,
    pub strategy: WaitStrategy
}

#[derive(Debug, Clone, Copy)]
pub enum WaitStrategy {
    None,

    /// Sleep unconditionally for the given duration
    Sleep(Duration),

    /// Wait until an address (main_module.base + offset) is readable or changed
    MonitorOffset(u64, Duration),
}

impl RunArgs {
    pub fn new() -> Result<Self> {

        let cipher_key = std::env::var("CIPHER_KEY")?.as_bytes().to_vec();
        let aes_xor_key = std::env::var("AES_XOR_KEY")?;
        let aes_xor_key = hex::decode(aes_xor_key)?;

        let exe_path = PathBuf::from(std::env::var("EXE_PATH")?);
        let game_path = PathBuf::from(std::env::var("GAME_PATH")?);
        let output_path = env::current_exe().unwrap().parent().unwrap().to_owned().join("output");
        
        let args_str = env::var("EXE_ARGS").unwrap_or_default();
        let mut exe_args: Vec<String> = vec![exe_path.to_string_lossy().to_string()];
        exe_args.extend(args_str.split(',').map(|s| s.to_string()));

        let strategy = get_wait_strategy_from_env()?;

        Ok(Self {
            cipher_key,
            aes_xor_key,
            output_path,
            exe_path,
            exe_args,
            game_path,
            strategy
        })
    }
}

pub fn get_wait_strategy_from_env() -> Result<WaitStrategy> {
    let strategy = env::var("WAIT_STRATEGY").unwrap_or_else(|_| "NONE".to_string());

    match strategy.to_uppercase().as_str() {
        "NONE" => Ok(WaitStrategy::None),

        "SLEEP" => {
            let secs: u64 = env::var("WAIT_DURATION")
                .context("WAIT_DURATION is required for SLEEP")?
                .parse()
                .context("WAIT_DURATION must be a valid integer")?;
            Ok(WaitStrategy::Sleep(Duration::from_secs(secs)))
        }

        "MONITOR" => {
            let offset_str = env::var("WAIT_OFFSET")
                .context("WAIT_OFFSET is required for MONITOR")?;
            let offset = u64::from_str_radix(offset_str.trim_start_matches("0x"), 16)
                .context("WAIT_OFFSET must be a valid hex number")?;

            let secs: u64 = env::var("WAIT_DURATION")
                .context("WAIT_DURATION is required for MONITOR")?
                .parse()
                .context("WAIT_DURATION must be a valid integer")?;

            Ok(WaitStrategy::MonitorOffset(offset, Duration::from_secs(secs)))
        }

        _ => Err(anyhow::anyhow!("Unknown WAIT_STRATEGY: {}", strategy)),
    }
}