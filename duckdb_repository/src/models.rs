use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{custom_duration::CustomDuration, npc_type::NpcType};

#[derive(Debug)]
pub struct Zone {
    pub id: u32,
    pub created_on: DateTime<Utc>,
    pub name: String,
}

#[derive(Debug)]
pub struct Player {
    pub id: Uuid,
    pub created_on: DateTime<Utc>,
    pub updated_on: DateTime<Utc>,
    pub name: String,
    pub class_id: u32,
    pub character_id: u64,
    pub last_gear_score: f32,
}

#[derive(Debug)]
pub struct Raid {
    pub id: Uuid,
    pub created_on: DateTime<Utc>,
    pub name: String,
    pub sub_name: Option<String>,
    pub gate: u8,
    pub zone_ids: Vec<u32>,
}

#[derive(Debug)]
pub struct Npc {
    pub id: Uuid,
    pub created_on: DateTime<Utc>,
    pub name: String,
    pub npc_id: u32,
    pub npc_type: NpcType,
    pub raid_id: Uuid,
}

#[derive(Debug)]
pub struct HpSession {
    pub id: Uuid,
    pub npc_id: Uuid,
    pub confrontation_id: Uuid,
    pub started_on: DateTime<Utc>,
    pub ended_on: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct HpLog {
    pub session_id: Uuid,
    pub recorded_on: DateTime<Utc>,
    pub value: i64
}

#[derive(Debug)]
pub struct Confrontation {
    pub id: Uuid,
    pub created_on: DateTime<Utc>,
    pub raid_id: Uuid,
    pub is_cleared: bool,
    pub total_damage_taken: i64,
    pub total_damage_dealt: i64,
    pub duration: CustomDuration,
}

#[derive(Debug)]
pub struct PlayerStats {
    pub player_id: Uuid,
    pub confrontation_id: Uuid,
    pub created_on: DateTime<Utc>,
    pub total_damage_taken: i64,
    pub total_damage_dealt: i64,
}