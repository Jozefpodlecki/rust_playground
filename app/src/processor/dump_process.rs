use std::path::PathBuf;

use anyhow::*;
use crate::{process::{ProcessDump, ProcessSnapshotter}, processor::ProcessorStep, types::LaunchMethod};

pub struct DumpProcessStep {
    exe_path: PathBuf,
    dump_path: PathBuf,
    dest_path: PathBuf,
    exe_args: Vec<String>,
    launch_method: LaunchMethod
}

impl ProcessorStep for DumpProcessStep {
    fn name(&self) -> String {
        format!("Dumping process {:?}", self.exe_path.file_name().unwrap())
    }

    fn can_execute(&self) -> bool {
        if !self.exe_path.is_file() {
            return false
        }

        if !self.dest_path.exists() {
            return false
        }

        if self.dump_path.exists() {
            return false
        }

        true
    }

    fn execute(self: Box<Self>) -> Result<()> {

        let snapshotter = ProcessSnapshotter::run(
            &self.exe_path,
            &self.exe_args,
            self.launch_method)?;
        let snapshot = snapshotter.get_snapshot()?;
        let dump = ProcessDump::save(snapshot, &self.dump_path)?;

        Ok(())
    }
}

impl DumpProcessStep {
    pub fn new(
        exe_path: PathBuf,
        dest_path: PathBuf,
        exe_args: Vec<String>,
        launch_method: LaunchMethod
    ) -> Self {
        let dump_path = ProcessDump::get_path(&exe_path, &dest_path);

        Self {
            exe_path,
            dump_path,
            dest_path,
            exe_args,
            launch_method
        }
    }
}