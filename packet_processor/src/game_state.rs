use crate::{models::{Boss, Player}, packet::{AttackPacket, NewNpcPacket, NewPlayerPacket}};

pub struct GameState {
    players: Vec<Player>,
    boss: Option<Boss>
}

impl GameState {
    pub fn new() -> Self {
        Self {
            players: vec![],
            boss: None,
        }
    }


    pub fn on_new_player(&self, packet: NewPlayerPacket) {

    }

    pub fn on_new_npc(&self, packet: NewNpcPacket) {
        
    }

    pub fn on_attack(&self, packet: AttackPacket) {
        
    }
}