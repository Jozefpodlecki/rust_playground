use std::time::Duration;

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Clone)]
pub enum Packet {
    NewPlayer {
        id: u64,
        character_id: u64,
        name: String,
    },
    Party {
        id: u32,
        members: Vec<PartyMember>,
    },
    NewBoss {
        id: u64,
        name: String,
    },
    Damage {
        skill_id: u32,
        source_id: u64,
        target_id: u64,
        value: u64
    },
    RaidEnd
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Clone)]
pub struct Player {
    pub id: u64,
    pub character_id: u64,
    pub name: String,
}

pub enum EntityType {
    Player,
    Boss
}

pub struct Entity {
    pub id: u64,
    pub kind: EntityType,
    pub character_id: u64,
    pub name: String,  
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Clone)]
pub struct PartyMember {
    pub character_id: u64,
    pub name: String,
}

#[derive(Default, Debug, Encode, Decode, Serialize, Deserialize, Clone)]
pub struct Boss {
    pub id: u64,
    pub name: String,
    pub hp: u64,
    pub current_hp: u64,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub port: u16,
    pub summary_emit_interval: Duration
}