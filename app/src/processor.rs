use std::{env, fs::{self, File}, io::Write, path::Path};

use anyhow::*;
use log::info;
use crate::types::{LpkEntryType, LpkInfo, RunArgs};

pub fn parse_ue3_object(args: RunArgs) -> Result<()> {


    Ok(())
}

pub fn extract_lpk(args: RunArgs) -> Result<()> {
    let RunArgs {
        lpk_dir,
        cipher_key,
        aes_xor_key,
        output_path
    } = args;

    let output_path = Path::new(&output_path);

    for entry in fs::read_dir(&lpk_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("lpk") {
            continue;
        }

        let lpk_name = path.file_stem().unwrap().to_str().unwrap();
        let output_path = output_path.join(lpk_name);
        info!("Creating directory {}", output_path.to_str().unwrap());
        fs::create_dir_all(&output_path)?;
        let file_path = path.to_str().unwrap();

        let mut lpk_info = LpkInfo::new(file_path, &cipher_key)?;

        for entry in lpk_info.entries.iter_mut() {
            match entry {
                LpkEntryType::BlowfishCompressed(entry) => {
                    let file_name = entry.file_name.to_owned();
                    let content = entry.get_content(&aes_xor_key)?;
                    save_to_disk(&file_name, &output_path, &content)?;
                },
                LpkEntryType::Aes256CbcEncrypted(entry) => {
                    let file_name = entry.file_name.to_owned();
                    let content = entry.get_content(&aes_xor_key)?;
                    save_to_disk(&file_name, &output_path, &content)?;
                },
            }
        }
    }

    Ok(())
}

pub fn save_to_disk(file_name: &str, output_path: &Path, content: &[u8]) -> Result<()> {
    let output_path = output_path.join(file_name);
    info!("Saving to {}", output_path.to_str().unwrap());

    let mut file = File::create(&output_path)?;
    file.write_all(&content)?;

    Ok(())
}