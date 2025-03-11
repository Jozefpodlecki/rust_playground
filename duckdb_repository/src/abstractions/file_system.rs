use std::{env, fs, path::PathBuf};

use anyhow::{Ok, Result};

pub struct FileSystem;

impl FileSystem {
    pub fn new() -> Self {
        Self
    }

    pub fn get_migration_files(&self) -> Result<Vec<PathBuf>> {
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

    pub fn get_migration_files_after(&self, last_migration: &str) -> Result<Vec<PathBuf>> {
        let migrations = self.get_migration_files()?;

        let filtered: Vec<PathBuf> = migrations.into_iter()
            .filter(|file_path| {
                let migration_name = file_path.file_name().unwrap().to_string_lossy().to_string();
                migration_name.as_str() > last_migration
            })
            .collect();

        Ok(filtered)
    }

    pub fn read_file(&self, path: &PathBuf) -> Result<String> {
        let content = fs::read_to_string(path)?;
        Ok(content)
    }
}
