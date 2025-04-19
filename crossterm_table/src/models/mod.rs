pub mod encounter_template;
pub mod player_template;
pub mod class;
pub mod player;
pub mod encounter;
pub mod boss_state;

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use class::Class;
pub use encounter::*;
pub use encounter_template::EncounterTemplate;
use player_template::{BuffTarget, BuffType};
pub use player_template::PlayerTemplate;
pub use player::*;
use uuid::Uuid;


pub struct BossTemplate {
    pub name: &'static str,
    pub max_hp: u64,
    pub hp_bars: u64
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
    pub instance_id: u32,
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
pub struct PartyState {
    pub active_buffs: HashMap<u32, Buff>,
}
