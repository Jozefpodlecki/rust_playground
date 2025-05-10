use crate::service::VerifierService;

pub struct ExerciseOneVerifierService;

impl VerifierService for ExerciseOneVerifierService {
    fn verify(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

impl ExerciseOneVerifierService {
    pub fn new() -> Self {
        Self {}
    }
}