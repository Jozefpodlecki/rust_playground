use std::path::PathBuf;

use anyhow::*;
use crate::{config::LaunchMethod, misc::export_dump::ExportDump, process::{create_dump_summary, ProcessDump, ProcessSnapshotter}, processor::ProcessorStep};

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

        let summary_path = self.dest_path.join("summary.json");
        create_dump_summary(&dump, summary_path)?;

        let export_file_path = self.dest_path.join("exports.json");
        let export_dump = ExportDump::create(&export_file_path, &dump.modules, &dump.exports)?;

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
        let file_name = exe_path.file_stem().unwrap().to_string_lossy().to_string();
        let dest_path = dest_path.join(file_name);
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