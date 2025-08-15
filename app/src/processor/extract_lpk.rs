use std::{collections::HashMap, fs::{self, File}, io::Write, path::{Path, PathBuf}};

use anyhow::*;
use log::info;
use crate::{misc::lpk::get_lpks, processor::ProcessorStep};

pub struct ExtractLpkStep {
    cipher_key: Vec<u8>,
    aes_xor_key: Vec<u8>,
    src_path: PathBuf,
    dest_path: PathBuf,
}

impl ProcessorStep for ExtractLpkStep {
    fn name(&self) -> String {
        format!("Extract Lpk in {:?}", self.src_path)
    }

    fn can_execute(&self) -> bool {
        if !self.src_path.exists() {
            return false
        }

        !fs::read_dir(&self.dest_path)
            .map(|entries| {
                entries
                    .flatten()
                    .any(|entry| entry.path().is_dir())
            })
            .unwrap_or(false)
    }

    fn execute(self: Box<Self>) -> Result<()> {
        
        let mut map: HashMap<String, Vec<String>> = HashMap::new();

        for mut lpk_info in get_lpks(
            &self.src_path,
            &self.cipher_key,
            &self.aes_xor_key)? {
 
            let lpk_name = lpk_info.file_path.file_stem().unwrap().to_str().unwrap();
            let output_path = self.dest_path.join(lpk_name);
            info!("Creating directory {}", output_path.to_str().unwrap());
            fs::create_dir_all(&output_path)?;

            let items = map.entry(lpk_name.to_string()).or_default();
            lpk_info.load()?;

            for entry in lpk_info.get_entries().iter_mut() {
                items.push(entry.metadata.file_path.clone());
                let content = entry.content.to_bytes()?;
                let file_path = &entry.metadata.file_path;
                let file_path = &output_path.join(file_path);
                
                fs::create_dir_all(&output_path.parent().unwrap())?;
                info!("Saving to {:?}", file_path.strip_prefix(&output_path));
                let mut file = File::create(&file_path)?;
                file.write_all(&content)?;
            }
        }

        let writer = File::create(self.dest_path.join("lpk_map.json"))?;
        serde_json::to_writer_pretty(writer, &map)?;

        Ok(())
    }
}

impl ExtractLpkStep {
    pub fn new(
        cipher_key: Vec<u8>,
        aes_xor_key: Vec<u8>,
        src_path: PathBuf,
        dest_path: PathBuf) -> Self {
        Self {
            cipher_key,
            aes_xor_key,
            src_path,
            dest_path
        }
    }
}

pub fn collect_lpk_paths<P: AsRef<Path>>(lpk_dir: P) -> Result<Vec<PathBuf>> {
    let mut lpk_paths = Vec::new();

    for entry in fs::read_dir(lpk_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("lpk") {
            lpk_paths.push(path);
        }
    }

    Ok(lpk_paths)
}