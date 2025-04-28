use std::time::Duration;

use bincode::{Decode, Encode};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Clone)]
pub enum Packet {
    NewPlayer {
        id: u64,
        character_id: u64,
        name: String,
        gear_score: f32
    },
    Party {
        id: u32,
        members: Vec<PartyMember>,
    },
    NewBoss {
        id: u64,
        name: String,
    },
    AddBuff {
        target_id: u64,
        effect: StatusEffect
    },
    AddPartyBuff {
        target_id: u64,
        effect: StatusEffect
    },
    Damage {
        skill_id: u32,
        source_id: u64,
        target_id: u64,
        value: u64,
        hp: u64,
        current_hp: u64
    },
    RaidEnd
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Clone)]
pub struct Player {
    pub id: u64,
    pub character_id: u64,
    pub name: String,
    pub gear_score: f32
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum EntityType {
    #[default]
    Unknown,
    Projectile,
    Player,
    Boss
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Entity {
    pub id: u64,
    pub kind: EntityType,
    pub character_id: Option<u64>,
    pub owner_id: Option<u64>,
    pub name: String,
    pub gear_score: f32
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EntityStats {
    pub total_damage: u64,
    pub dps: f32,
    pub total_damage_percentage: f32,
    pub current_hp: u64,
    pub hp: u64
}

#[derive(Debug, Serialize)]
pub struct EncounterFragment<'a> {
    pub id: &'a Uuid,
    pub started_on: &'a DateTime<Utc>,
    pub players: Vec<EntityStatsSummary>
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EntityStatsSummary {

}

#[derive(Debug, Default)]
pub struct RaidStats {
    pub total_damage: u64,
    pub dps: f32
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Clone)]
pub struct PartyMember {
    pub character_id: u64,
    pub name: String,
    pub gear_score: f32
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

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum BuffTarget {
    TargetSelf,
    Party
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Clone)]
pub struct StatusEffect {
    pub source_id: u64,
    pub target: BuffTarget,
    pub duration: u64,
    pub value: u64
}