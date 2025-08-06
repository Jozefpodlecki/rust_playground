use std::{env, fs::{self, File}, io::{BufWriter, Cursor, Read, Seek, Write}, path::{Path, PathBuf}};

use anyhow::*;
use byteorder::{LittleEndian, ReadBytesExt};
use log::info;
use crate::{lpk::{get_lpks, LpkInfo}, process_dumper::{self, ProcessDumper}, processor::ProcessorStep, types::{RunArgs, WaitStrategy}};

pub struct DumpProcessStep {
    exe_path: PathBuf,
    dest_path: PathBuf,
    exe_args: Vec<String>,
    strategy: WaitStrategy
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

        true
    }

    fn execute(self: Box<Self>) -> Result<()> {

        let mut process_dumper = ProcessDumper::new(&self.exe_path, &self.dest_path)?;

        process_dumper.run_or_get_cached(&self.exe_args, self.strategy);

        Ok(())
    }
}

impl DumpProcessStep {
    pub fn new(
        exe_path: PathBuf,
        dest_path: PathBuf,
        exe_args: Vec<String>,
        strategy: WaitStrategy
    ) -> Self {
        Self {
            exe_path,
            dest_path,
            exe_args,
            strategy
        }
    }
}