use std::{fs::{self, File}, io::Write, path::{Path, PathBuf}, time::SystemTime};

use anyhow::*;
use chrono::Local;
use log::info;
use crate::{processor::ProcessorStep, sql_migrator::*, config::AppConfig};

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
            output_path,
            cipher_key,
            aes_xor_key,
            ..
        } = self.config;

        let output_db_path = output_path.join("db");
        fs::create_dir_all(&output_db_path)?;

        let lpk_path = output_path.join("lpk");
        let sqlite_dir = lpk_path.join(r"data2\EFGame_Extra\ClientData\TableData");
        let jss_sqlite_dir = lpk_path.join(r"data2\EFGame_Extra\ClientData\TableData\jss");
       
        if let Some(duckdb_path) = get_latest_db(&output_db_path) {
            info!("Using latest db");
            let duck_db = DuckDb::new(&duckdb_path)?;

            // let records = duck_db.get_table_data("data.SkillEffect")?;
            // let file_path = output_db_path.join("skill_effect.json");
            // let writer = File::create(file_path)?;
            // serde_json::to_writer_pretty(writer, &records)?;

            // let records = duck_db.get_table_data("data.Npc")?;
            // let file_path = output_db_path.join("npc.json");
            // let writer = File::create(file_path)?;
            // serde_json::to_writer_pretty(writer, &records)?;

            // let records = duck_db.get_table_data("data.Skill")?;
            // let file_path = output_db_path.join("skill.json");
            // let writer = File::create(file_path)?;
            // serde_json::to_writer_pretty(writer, &records)?;

            // let records = duck_db.get_table_data("data.SkillBuff")?;
            // let file_path = output_db_path.join("skill_buff.json");
            // let writer = File::create(file_path)?;
            // serde_json::to_writer_pretty(writer, &records)?;

            // let records = duck_db.get_table_data("data.Item")?;
            // let file_path = output_db_path.join("item.json");
            // let writer = File::create(file_path)?;
            // serde_json::to_writer_pretty(writer, &records)?;
        }
        else
        {
            let duckdb_path = create_new_db_file(&output_db_path);

            let merger = DbMerger::new(&duckdb_path, 1000)?;
            merger.setup()?;

            merger.create_enums(&lpk_path)?;
            merger.merge_data(sqlite_dir)?;
            merger.merge_jss(jss_sqlite_dir)?;

            merger.insert_lpk_metadata(&lpk_path, cipher_key, aes_xor_key)?;
            let summary = merger.post_work()?;

            let summary_path = output_db_path.join("refactor.sql");
            save_summary_script(summary, &summary_path)?;
        }

        // merger.insert_loa_data(&lpk_path)?;

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

pub fn save_summary_script(summary: String, file_path: &Path) -> Result<()>  {
    let mut file = File::create(file_path)?;
    file.write_all(summary.as_bytes())?;
    Ok(())
}

pub fn get_latest_db(output_path: &Path) -> Option<PathBuf> {
    let mut latest: Option<(SystemTime, PathBuf)> = None;

    for entry in fs::read_dir(&output_path).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();

        if path.extension().and_then(|ext| ext.to_str()) != Some("duckdb") {
            continue;
        }

        let metadata = entry.metadata().ok()?;
        let created = metadata.created().or_else(|_| metadata.modified()).ok()?;

        match &latest {
            Some((latest_time, _)) if created <= *latest_time => {}
            _ => latest = Some((created, path)),
        }
    }

    latest.map(|(_, path)| path)
}

pub fn create_new_db_file(dest_path: &Path) -> PathBuf {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S_%3f").to_string();
    let file_name = format!("output_{}.duckdb", timestamp);
    let duckdb_path = dest_path.join(file_name);

    duckdb_path
}