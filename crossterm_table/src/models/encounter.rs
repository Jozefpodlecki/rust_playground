use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::Player;

#[derive(Default, Debug, Clone)]
pub struct Boss {
    pub id: u64,
    pub name: &'static str,
    pub max_hp: u64,
    pub current_hp: u64,
    pub hp_percentage: f32,
    pub max_hp_bars: u64,
    pub hp_bars: f32,
    pub bar_per_hp: f32
}


#[derive(Default, Debug, Clone)]
pub struct PartyStats {
    pub dps: u64,
    pub total_damage: u64,
    pub total_damage_percentage: f32
}

#[derive(Default, Debug, Clone)]
pub struct Party {
    pub id: u64,
    pub players: Vec<Player>,
    pub stats: PartyStats
}

#[derive(Default, Debug, Clone)]
pub struct Encounter {
    pub id: Uuid,
    pub boss: Boss,
    pub duration: EncounterDuration,
    pub started_on: DateTime<Utc>,
    pub parties: Vec<Party>,
    pub stats: EncounterStats
}

#[derive(Default, Debug, Clone)]
pub struct EncounterStats {
    pub dps: u64,
    pub total_damage: u64,
    pub ttk: String,
}

#[derive(Default, Debug, Clone)]
pub struct EncounterDuration {
    pub elapsed_seconds: i64,
    pub mmss: String,
}
