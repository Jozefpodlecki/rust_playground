use std::{env, fs, path::PathBuf};

use chrono::Utc;
use duckdb::{params, Connection, DuckdbConnectionManager};
use r2d2::Pool;
use anyhow::{Ok, Result};

use crate::abstractions::FileSystem;

use super::repositories::ConfigRepository;

pub struct MigrationRunner {
    pool: Pool<DuckdbConnectionManager>,
    config_repository: ConfigRepository,
    file_system: FileSystem
}

impl MigrationRunner {
    pub fn new(
        pool: Pool<DuckdbConnectionManager>,
        config_repository: ConfigRepository,
        file_system: FileSystem
    ) -> Self {
        Self { 
            pool,
            config_repository,
            file_system
        }
    }

    pub fn run(&self, version: &str) -> Result<()> {

        let connection = self.pool.get()?;

        if !Self::table_exists(&connection, "Config")? {
            self.apply_all_migrations(version)?;
        } else {
            self.apply_new_migrations(version)?;
        }


        Ok(())
    }

    fn apply_all_migrations(&self, version: &str) -> Result<()> {
        let connection = self.pool.get()?;
        let migrations = self.get_migration_files()?;
        let mut last_migration = None;

        for file_path in migrations {
            let content = fs::read_to_string(&file_path)?;
            connection.execute_batch(&content)?;
            let migration_name = file_path.file_name().unwrap().to_string_lossy().to_string();
            last_migration = Some(migration_name);
        }

        if let Some(last_migration) = last_migration {
            self.update_config_table(version, &last_migration)?;
        }

        Ok(())
    }

    fn apply_new_migrations(&self, version: &str) -> Result<()> {

        let connection = self.pool.get()?;
        let last_applied_migration = self.get_last_applied_migration(&connection)?;
        let migrations = self.get_migration_files_after(&last_applied_migration.unwrap())?;
        let mut last_migration = None;

        for file_path in migrations {
            let content = fs::read_to_string(&file_path)?;
            connection.execute_batch(&content)?;
            let migration_name = file_path.file_name().unwrap().to_string_lossy().to_string();
            last_migration = Some(migration_name);
        }

        if let Some(last_migration) = last_migration {
            self.update_config_table(version, &last_migration)?;
        }

        Ok(())
    }

    fn update_config_table(&self, version: &str, migration: &str) -> Result<()> {
        let connection = self.pool.get()?;
        let sql = r"
        INSERT INTO Config
        (version, last_migration, updated_on)
        VALUES
        (?, ?, ?)
        ON CONFLICT(version) DO UPDATE SET
            last_migration = excluded.last_migration,
            updated_on = excluded.updated_on
        ";

        let current_timestamp = Utc::now().to_string();
        let mut statement = connection.prepare(sql)?;
        let params = params![version, migration, current_timestamp];
        let _ = statement.execute(params)?;

        Ok(())
    }

    fn get_migration_files_after(&self, last_migration: &str) -> Result<Vec<PathBuf>> {
        let migrations = self.get_migration_files()?;

        let filtered: Vec<PathBuf> = migrations.into_iter()
            .filter(|file_path| {
                let migration_name = file_path.file_name().unwrap().to_string_lossy().to_string();
                migration_name.as_str() > last_migration
            })
            .collect();

        Ok(filtered)
    }

    fn get_migration_files(&self) -> Result<Vec<PathBuf>> {
        let executable_path = if cfg!(test) {
            env::current_exe()?.parent().unwrap().to_path_buf()
        } else {
            env::current_exe()?
        };

        let executable_directory = executable_path.parent().unwrap();
        let migrations_directory = executable_directory.join("migrations");

        let mut files: Vec<_> = fs::read_dir(migrations_directory)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map_or(false, |ext| ext == "sql"))
            .collect();

        files.sort();
        Ok(files)
    }

    fn get_last_applied_migration(&self, connection: &Connection) -> Result<Option<String>> {
        let sql = "SELECT last_migration FROM Config LIMIT 1";

        let mut statement = connection.prepare(sql)?;
        let result = statement.query_row([], |row| row.get(0)).ok();

        Ok(result)
    }

    fn table_exists(connection: &Connection, name: &str) -> Result<bool> {
        let sql = r"
        SELECT
            EXISTS (SELECT 1 FROM duckdb_tables WHERE table_name = ?)
        ";
    
        let mut statement = connection.prepare(sql)?;
        let params = [name];
        let result = statement.query_row(params, |row| row.get(0))?;
        
        Ok(result)
    }
}