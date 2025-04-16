use std::collections::HashMap;

use super::class::Class;

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
    pub total_damage_percentage: f32,
    pub hyper_awakening_damage: u64,
    pub damage_with_brand: u64,
    pub damage_with_attack_power_buff: u64,
    pub damage_with_identity_buff: u64,
    pub damage_with_hat_buff: u64,
    pub brand_percentage: f32,
    pub hat_percentage: f32,
    pub attack_power_buff_percentage: f32,
    pub identity_percentage: f32,
    pub skills: PlayerSkillsStats
}

#[derive(Default, Clone, Debug)]
pub struct PositionalStats {
    pub damage: u64,
    pub count: u64,
}

#[derive(Default, Clone, Debug)]
pub struct PlayerSkillStats {
    pub hit_count: u64,
    pub crit_count: u64,
    pub back_attack: PositionalStats,
    pub front_attack: PositionalStats
}

#[derive(Default, Clone, Debug)]
pub struct PlayerSkillsStats {
    pub skill: HashMap<u32, PlayerSkillStats>,
    pub hit_count: u64,
    pub crit_count: u64
}