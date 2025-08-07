

use std::{collections::{HashMap, HashSet}, fs::{self}, path::PathBuf};
use log::*;
use anyhow::*;

use crate::processor::ProcessorStep;

pub struct CleanupDirectoryStep {
    path: PathBuf,
    files: HashSet<String>,
    folders: HashSet<String>
}

impl ProcessorStep for CleanupDirectoryStep {
    fn name(&self) -> String {
        String::from("Cleanup directory")
    }

    fn can_execute(&self) -> bool {
        true
    }

    fn execute(self: Box<Self>) -> Result<()> {
        
        let items: Vec<_> = fs::read_dir(&self.path)?
            .filter_map(Result::ok)
            .map(|e| e.path())
            .collect();

        // info!("{:?}", items);
        // info!("{:?}", self.patterns);

        for item in items {
            let path_str = item.to_string_lossy();

            for pattern in &self.files {
                if path_str.contains(pattern) {
                    if item.is_file() {
                        fs::remove_file(&item)?;
                    }  
                }
            }
            for pattern in &self.folders {
                if path_str.contains(pattern) {
                    if item.is_dir() {
                        fs::remove_dir_all(&item)?;
                    }
                }
            }
        }
        
        Ok(())
    }
}

impl CleanupDirectoryStep {
    pub fn new(path: PathBuf, files: HashSet<String>, folders: HashSet<String>) -> Self {
        Self {
            path,
            files,
            folders
        }
    }
}