pub mod sorceress;
pub mod aeromancer;
pub mod paladin;
pub mod artist;
pub mod artillerist;
pub mod bard;
pub mod generic;
pub mod builder;

pub use builder::PlayerTemplateBuilder;

use chrono::{DateTime, Duration, Utc};

use super::Class;

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

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub enum BuffCategory {
    #[default]
    Buff,
    Debuff
}

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub enum BuffTarget {
    #[default]
    TargetSelf,
    Party
}

#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum BuffType {
    #[default]
    Brand,
    AttackPowerBuff,
    Identity,
    DamageAmplification,
    HyperAwakeningTechnique,
    HyperAwakeningTechniqueOutgoingDamage,
    Shield,
    DamageReduction,
    Other
}
#[derive(Default, Debug, Clone)]
pub struct BuffTemplate {
    pub category: BuffCategory,
    pub target: BuffTarget,
    pub kind: BuffType,
    pub duration: Duration,
    pub value: f32
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
    pub buffs: Vec<BuffTemplate>
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

#[derive(Default, Debug, Clone)]
pub struct PlayerTemplate {
    pub name: Option<String>,
    pub class: Class,
    pub crit_rate: f32,
    pub cooldown_reduction: f32,
    pub attack_power: u64,
    pub crit_damage: f32,
    pub skills: Vec<SkillTemplate>,
}