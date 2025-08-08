use anyhow::*;

pub struct Cfg {

}
pub struct CfgBuilder {

}

impl CfgBuilder {
    pub fn new() -> Self {
        Self {

        }
    }

    pub fn build(&mut self, data: Vec<u64>) -> Result<Cfg> {
        Ok(Cfg {})
    }
} 