

use std::{collections::{HashMap, HashSet}, fs::{self, File}, path::PathBuf};
use log::*;
use anyhow::*;

use crate::processor::ProcessorStep;

pub struct CleanupDirectoryStep {
    path: PathBuf,
    patterns: Vec<String>
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
            .map(|e| e.path().to_string_lossy().to_string())
            .collect();

        // info!("{:?}", items);
        // info!("{:?}", self.patterns);

        for item in items {
            for pattern in &self.patterns {
                if item.contains(pattern) {
                    info!("Would remove {}", item);
                }
            }   
        }
        
        Ok(())
    }
}

impl CleanupDirectoryStep {
    pub fn new(path: PathBuf, patterns: Vec<String>) -> Self {
        Self {
            path,
            patterns
        }
    }
}