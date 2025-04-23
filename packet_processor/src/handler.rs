use anyhow::*;
use bincode::config::Configuration;
use crate::{app_state::AppState, models::Packet};

pub struct Handler {
    config: Configuration
}

impl Handler {
    pub fn new() -> Self {
        let config = bincode::config::standard();

        Self {
            config
        }
    }

    pub async fn handle(&self, data: &[u8], state: &mut AppState) -> Result<()> {
        
        let (packet, _) = bincode::decode_from_slice::<Packet, Configuration>(data, self.config)?;
        
        match packet {
            Packet::NewPlayer { id, name } => state.new_player(id, name),
            Packet::Party { id, members } => todo!(),
            Packet::NewBoss { id, name } => todo!(),
            Packet::Damage { skill_id, source_id, target_id, value } => todo!(),
            Packet::RaidEnd => todo!(),
        }
        
        anyhow::Ok(())
    }
}