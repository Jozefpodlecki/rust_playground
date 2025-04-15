pub mod encounter_template;
pub mod player_template;
pub mod class;
pub mod player;


use std::collections::HashMap;

use chrono::{DateTime, Utc};

use class::Class;
pub use encounter_template::EncounterTemplate;
pub use player_template::PlayerTemplate;
pub use player::*;

#[derive(Default, Debug, Clone)]
pub struct PartyStats {
    pub dps: u64,
    pub total_damage: u64,
}

#[derive(Default, Debug, Clone)]
pub struct Party {
    pub id: u64,
    pub players: Vec<Player>,
    pub stats: PartyStats
}

#[derive(Default, Debug, Clone)]
pub struct Encounter {
    pub boss: Boss,
    pub duration: EncounterDuration,
    pub ttk: String,
    pub started_on: DateTime<Utc>,
    pub parties: Vec<Party>,
    pub stats: EncounterStats
}

#[derive(Default, Debug, Clone)]
pub struct EncounterStats {
    pub dps: u64,
    pub total_damage: u64,
}

#[derive(Default, Debug, Clone)]
pub struct EncounterDuration {
    pub elapsed_seconds: u64,
    pub mmss: String,
}

pub struct BossTemplate {
    pub name: &'static str,
    pub max_hp: u64,
    pub hp_bars: u64
}

#[derive(Default, Debug, Clone)]
pub struct Boss {
    pub id: u64,
    pub name: &'static str,
    pub max_hp: u64,
    pub current_hp: u64,
    pub hp_percentage: f32,
    pub hp_bars: u64
}

pub struct AttackResult {
    pub skill_id: u32,
    pub damage: u64,
    pub is_critical: bool,
}

