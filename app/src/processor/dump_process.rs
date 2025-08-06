use std::{env, fs::{self, File}, io::{BufWriter, Cursor, Read, Seek, Write}, path::{Path, PathBuf}};

use anyhow::*;
use byteorder::{LittleEndian, ReadBytesExt};
use log::*;
use crate::{lpk::{get_lpks, LpkInfo}, process_dumper::{self, ProcessDumper}, processor::ProcessorStep, types::{LaunchMethod}};

pub struct DumpProcessStep {
    exe_path: PathBuf,
    output_bin_path: PathBuf,
    dest_path: PathBuf,
    exe_args: Vec<String>,
    launch_ethod: LaunchMethod
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

        if self.output_bin_path.exists() {
            return false
        }

        true
    }

    fn execute(self: Box<Self>) -> Result<()> {

        let mut process_dumper = ProcessDumper::new(&self.exe_path, &self.output_bin_path)?;

        let dump = process_dumper.run(&self.exe_args, self.launch_ethod)?;

        // for block in dump.blocks.iter().filter(|pr| pr.block.is_executable) {
        //     info!("Executable 0x{:X}", block.block.base);
        // }

        Ok(())
    }
}

impl DumpProcessStep {
    pub fn new(
        exe_path: PathBuf,
        dest_path: PathBuf,
        exe_args: Vec<String>,
        launch_ethod: LaunchMethod
    ) -> Self {
        let output_bin_path = ProcessDumper::get_bin_path(&exe_path, &dest_path);

        Self {
            exe_path,
            output_bin_path,
            dest_path,
            exe_args,
            launch_ethod
        }
    }
}