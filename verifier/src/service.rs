use anyhow::*;

pub trait VerifierService {
    fn verify(&self) -> Result<()>;
}


