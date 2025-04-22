use anyhow::*;
use serde::Serialize;

pub struct Emitter {

}

impl Emitter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn emit<T: Serialize>(&self, data: T) -> Result<()> {
        anyhow::Ok(())
    }
}