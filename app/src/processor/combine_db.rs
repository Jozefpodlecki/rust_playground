use std::{env, fs::{self, File}, io::{BufWriter, Cursor, Read, Seek, Write}, path::{Path, PathBuf}};

use anyhow::*;
use byteorder::{LittleEndian, ReadBytesExt};
use chrono::Local;
use log::info;
use crate::{lpk::{get_lpks, LpkInfo}, processor::ProcessorStep, sql_migrator::*, types::AppConfig};

pub struct CombineDbStep {
    config: AppConfig
}

impl ProcessorStep for CombineDbStep {
    fn name(&self) -> String {
        format!("Combine databases")
    }

    fn can_execute(&self) -> bool {
        if !self.config.output_path.exists() {
            return false
        }

        true
    }

    fn execute(self: Box<Self>) -> Result<()> {

        let AppConfig {
            aes_xor_key,
            cipher_key,
            exe_paths,
            output_path,
            ..
        } = self.config;

        let lpk_path = output_path.join("lpk");
        let sqlite_dir = lpk_path.join(r"data2\EFGame_Extra\ClientData\TableData");
        let jss_sqlite_dir = lpk_path.join(r"data2\EFGame_Extra\ClientData\TableData\jss");
       
        let duckdb_path = create_new_db_file(&output_path);

        let merger = DbMerger::new(&duckdb_path, 1000)?;
        merger.setup();

        for exe_info in exe_paths {
            merger.insert_assembly(exe_info.path, &output_path);
        }

        
        // merger.create_enums(&lpk_path)?;
        // merger.merge_data(sqlite_dir)?;
        // merger.merge_jss(jss_sqlite_dir)?;
        // merger.insert_lpk_metadata(&lpk_path, self.cipher_key, self.aes_xor_key);
        // merger.insert_loa_data(&lpk_path);

        // let summary = merger.post_work()?;

        // let summary_path = self.dest_path.join("refactor.sql");
        // let mut file = File::create(summary_path)?;
        // file.write_all(summary.as_bytes())?;

        Ok(())
    }
}

impl CombineDbStep {
    pub fn new(config: AppConfig) -> Self {

        Self {
            config
        }
    }
}


pub fn create_new_db_file(dest_path: &Path) -> PathBuf {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S_%3f").to_string();
    let file_name = format!("output_{}.duckdb", timestamp);
    let duckdb_path = dest_path.join(file_name);

    duckdb_path
}