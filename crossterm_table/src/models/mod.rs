pub mod encounter_template;
pub mod player_template;
pub mod class;

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use class::Class;
pub use encounter_template::EncounterTemplate;
pub use player_template::PlayerTemplate;


#[derive(Default, Debug, Clone)]
pub struct Party {
    pub id: u64,
    pub players: Vec<Player>,
}

#[derive(Default, Debug, Clone)]
pub struct Encounter {
    pub boss: Boss,
    pub duration: EncounterDuration,
    pub started_on: DateTime<Utc>,
    pub parties: Vec<Party>,
    pub stats: EncounterStats
}

#[derive(Default, Debug, Clone)]
pub struct EncounterStats {
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

#[derive(Default, Clone, Debug)]
pub struct Skill {
    pub id: u32,
    pub name: String
}

#[derive(Default, Clone, Debug)]
pub struct Player {
    pub id: u64,
    pub name: String,
    pub class: Class,
    pub stats: PlayerStats,
    pub skills: HashMap<u32, Skill>,
}

#[derive(Default, Clone, Debug)]
pub struct PlayerStats {
    pub crit_rate: f32,
    pub crit_damage: u64,
    pub dps: u64,
    pub total_damage: u64,
    pub damage_with_brand: u64,
    pub damage_with_attack_power_buff: u64,
    pub damage_with_identity_buff: u64,
    pub damage_with_hat_buff: u64,
    pub brand_percentage: f32,
    pub hat_percentage: f32,
    pub attack_power_buff_percentage: f32,
    pub identity_percentage: f32,
    pub skills: PlayerSkillStats
}

#[derive(Default, Clone, Debug)]
pub struct PlayerSkillStats {

}