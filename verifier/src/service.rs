use anyhow::*;

pub trait VerifierService {
    fn verify(&self, project_path: &str) -> Result<()>;
}


