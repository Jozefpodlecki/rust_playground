pub mod encounter_template;
pub mod player_template;
pub mod class;
pub mod player;


use std::collections::HashMap;

use chrono::{DateTime, Utc};

use class::Class;
pub use encounter_template::EncounterTemplate;
use player_template::{BuffTarget, BuffType};
pub use player_template::PlayerTemplate;
pub use player::*;
use uuid::Uuid;

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
    pub max_hp_bars: u64,
    pub hp_bars: f32,
    pub bar_per_hp: f32
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum HitOption {
    #[default]
    None,
    Back,
    Frontal,
    Flank,
}

#[derive(Default, Debug, Clone)]
pub struct AttackResult {
    pub source_id: u64,
    pub target_id: u64,
    pub hit_option: HitOption,
    pub skill_id: u32,
    pub damage: u64,
    pub is_critical: bool,
    pub is_hyper_awakening: bool,
    pub with_brand: bool,
    pub with_attack_power_buff: bool,
    pub with_identity_buff: bool,
    pub with_hat_buff: bool,
}

#[derive(Clone, PartialEq)]
pub struct Buff {
    pub target: BuffTarget,
    pub kind: BuffType,
    pub expires_on: DateTime<Utc>,
    pub value: f32
}

#[derive(Default)]
pub struct PlayerState {
    pub skill_cooldowns: HashMap<u32, DateTime<Utc>>,
    pub active_buffs: HashMap<u32, Buff>,
    pub identity: f32
}

#[derive(Default)]
pub struct BossState {
    pub id: u64,
    pub current_hp: u64,
    pub active_debuffs: HashMap<u32, Buff>,
}

#[derive(Default)]
pub struct PartyState {
    pub active_buffs: HashMap<u32, Buff>,
}
