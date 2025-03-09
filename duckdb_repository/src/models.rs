use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct Player {
    pub id: Uuid,
    pub class_id: u32,
    pub character_id: u64,
    pub last_gear_score: f32,
    pub name: String,
    pub created_on: DateTime<Utc>,
    pub updated_on: DateTime<Utc>,
}

pub struct Npc {
    pub id: Uuid,
    pub created_on: DateTime<Utc>,
    pub name: String,
    pub npc_type_id: u32,
    pub raid_id: Uuid,
}

pub struct HpSession {
    pub id: Uuid,
    pub recorded_on: DateTime<Utc>,
    pub entity_id: Uuid,
    pub raid_id: Uuid,
    pub started_on: DateTime<Utc>,
    pub ended_on: DateTime<Utc>,
}

pub struct HpLog {
    pub session_id: Uuid,
    pub recorded_on: DateTime<Utc>,
    pub value: i64
}

pub struct Confrontation {
    pub id: Uuid,
    pub created_on: DateTime<Utc>,
    pub raid_id: i64,
    pub is_cleared: bool,
    pub total_damage_taken: i64,
    pub total_damage_dealt: i64,
    pub duration: String,
}

pub struct PlayerStats {
    pub player_id: Uuid,
    pub confrontation_id: Uuid,
    pub created_on: DateTime<Utc>,
    pub total_damage_taken: i64,
    pub total_damage_dealt: i64,
}