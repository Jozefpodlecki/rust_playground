use anyhow::*;

mod copy_files;
mod extract_lpk;
mod decrypt_upk;
mod dump_process;
mod combine_db;
mod disassemble_process;
mod cleanup_directory;
mod parse_dump;
mod extract_pe;

pub use copy_files::CopyFileStep;
pub use extract_lpk::ExtractLpkStep;
pub use decrypt_upk::DecryptUpkStep;
pub use dump_process::DumpProcessStep;
pub use disassemble_process::DisassembleProcessStep;
pub use cleanup_directory::CleanupDirectoryStep;
pub use parse_dump::ParseDumpStep;
pub use extract_pe::ExtractPeStep;
pub use combine_db::CombineDbStep;

use log::info;

pub trait ProcessorStep {
    fn name(&self) -> String;
    fn can_execute(&self) -> bool;
    fn execute(self: Box<Self>) -> Result<()>;
}

pub struct Processor {
    steps: Vec<Box<dyn ProcessorStep>>
}

impl Processor {
    pub fn new() -> Self {
        Self {
            steps: vec![]
        }
    }

    pub fn add_step(&mut self, step: Box<dyn ProcessorStep>) {
        info!("Added step {}", step.name());
        self.steps.push(step);
    }

    pub fn run(self) -> Result<()> {
        for step in self.steps {
            
            if step.can_execute() {
                info!("Executing step: {}", step.name());
                step.execute()?;
            }
            else {
                info!("Skipping step: {}", step.name());
            }
        }

        Ok(())
    }
}