use anyhow::*;
use crate::app_state::AppState;

pub struct Handler {

}

impl Handler {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn handle(&self, data: &[u8], state: &mut AppState) -> Result<()> {
        anyhow::Ok(())
    }
}