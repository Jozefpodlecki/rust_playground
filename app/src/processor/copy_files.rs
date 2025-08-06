use std::{fs, path::PathBuf};
use log::*;
use walkdir::WalkDir;
use crate::processor::ProcessorStep;
use anyhow::*;

pub struct CopyFileStep<'a> {
    src_path: PathBuf,
    dest_path: PathBuf,
    extension: &'a str,
    recursive: bool,
}

impl<'a> ProcessorStep for CopyFileStep<'a> {
    fn name(&self) -> String {
        format!("Copy files from {:?} to {:?}", self.src_path, self.dest_path)
    }

    fn can_execute(&self) -> bool {

        if self.dest_path.exists() {
            return false;
        }

        if !self.src_path.is_dir() {
            return false
        }

        self
            .src_path
            .read_dir()
            .map(|mut dir| dir.any(|entry| {
                entry
                    .as_ref()
                    .map(|e| {
                        e.path()
                            .extension()
                            .map(|ext| ext == "lpk")
                            .unwrap_or(false)
                    })
                    .unwrap_or(false)
            }))
            .unwrap_or(false)
    }

    fn execute(self: Box<Self>) -> Result<()> {
        let _ = fs::create_dir_all(&self.dest_path);

        let entries: Box<dyn Iterator<Item = PathBuf>> = if self.recursive {
            Box::new(
                WalkDir::new(&self.src_path)
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter(|e| e.file_type().is_file())
                    .map(|e| e.path().to_path_buf()),
            )
        } else {
            Box::new(
                fs::read_dir(&self.src_path)?
                    .filter_map(Result::ok)
                    .map(|e| e.path()),
            )
        };

        for path in entries {
            if path.extension().map(|ext| ext == self.extension).unwrap_or(false) {
                let relative_path = path.strip_prefix(&self.src_path)?;
                let dest_file = self.dest_path.join(relative_path);

                if let Some(parent) = dest_file.parent() {
                    fs::create_dir_all(parent)?;
                }

                if let Err(e) = fs::copy(&path, &dest_file) {
                    warn!("Failed to copy {:?} to {:?}: {}",
                        path.file_name().unwrap().to_str().unwrap(),
                        dest_file.strip_prefix(&self.dest_path),
                        e);
                } else {
                    info!("Copied: {} -> {:?}",
                        path.file_name().unwrap().to_str().unwrap(),
                        dest_file.strip_prefix(&self.dest_path));
                }
            }
        }

        Ok(())
    }
}

impl<'a> CopyFileStep<'a> {
    pub fn new(
        src_path: PathBuf,
        dest_path: PathBuf,
        extension: &'a str,
        recursive: bool) -> Self {
        Self {
            src_path,
            dest_path,
            extension,
            recursive
        }
    }
}