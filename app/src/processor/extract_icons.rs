use std::{fs::{self, File}, io::BufReader, path::PathBuf};

use anyhow::*;
use crate::{processor::ProcessorStep};

pub struct ExtractIconsStep {
    src_path: PathBuf,
    dest_path: PathBuf,
}

impl ProcessorStep for ExtractIconsStep {
    fn name(&self) -> String {
        format!("Extract icons in {:?}", self.src_path)
    }

    fn can_execute(&self) -> bool {
        true
    }

    fn execute(self: Box<Self>) -> Result<()> {
        
        let icon_file = r"ReleasePC\Packages\OVSG0AE9W7OG8W2OD62YWO.upk";
        let icon_file = r"ReleasePC\Packages\VC2NXN2NA2H00FGHEHYEB00X.upk";
        let icon_path = self.dest_path.join(icon_file);

        let file = File::open(icon_path)?;
        let mut reader = BufReader::new(file);

        Ok(())
    }
}

impl ExtractIconsStep {
    pub fn new(
        src_path: PathBuf,
        dest_path: PathBuf) -> Self {
        Self {
            src_path,
            dest_path
        }
    }
}
