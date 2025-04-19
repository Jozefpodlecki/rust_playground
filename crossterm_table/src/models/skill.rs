use chrono::{DateTime, Duration, Utc};

use super::BuffTemplate;


#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum SkillType {
    #[default]
    Normal,
    Synergy,
    Brand,
    AttackPowerBuff,
    Identity,
    HyperAwakeningTechnique,
    Awakening,
    HyperAwakening,
}

#[derive(Default, Debug, Clone)]
pub struct SkillTemplate {
    pub id: u32,
    pub name: &'static str,
    pub min_ratio: f32,
    pub max_ratio: f32,
    pub identity_gain: f32,
    pub requires_identity: bool,
    pub kind: SkillType,
    pub priority: u8,
    pub cast_duration: Duration,
    pub cooldown: Duration,
    pub cooldown_gem: f32,
    pub cooldown_reduction: f32,
    pub initial_cooldown: Option<Duration>,
    pub buffs: Vec<BuffTemplate>,
    pub shared_skill_id: Option<u32>
}

#[derive(Default, Debug, Clone)]
pub struct Skill {
    pub id: u32,
    pub name: String,
    pub ready_on: DateTime<Utc>,
    pub min_ratio: f32,
    pub max_ratio: f32,
    pub identity_gain: f32,
    pub requires_identity: bool,
    pub kind: SkillType,
    pub priority: u8,
    pub cast_duration: Duration,
    pub cooldown: Duration,
    pub cooldown_gem: f32,
    pub cooldown_reduction: f32,
    pub initial_cooldown: Option<Duration>,
    pub buffs: Vec<BuffTemplate>
}