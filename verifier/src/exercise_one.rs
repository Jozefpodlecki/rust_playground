use std::process::Command;

use crate::service::VerifierService;

pub struct ExerciseOneVerifierService;

impl VerifierService for ExerciseOneVerifierService {
    fn verify(&self, project_path: &str) -> anyhow::Result<()> {

        let status = Command::new(project_path)
            .arg("some_argument")
            .status()
            .expect("Failed to run binary");

        if !status.success() {
            eprintln!("The command failed.");
            // exit(1);
        } else {
            println!("Successfully ran the binary.");
        }

        Ok(())
    }
}

impl ExerciseOneVerifierService {
    pub fn new() -> Self {
        Self {}
    }
}