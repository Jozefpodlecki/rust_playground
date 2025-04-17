pub mod berserker;
pub mod reaper;
pub mod destroyer;
pub mod gunslinger;
pub mod sorceress;
pub mod wardancer;
pub mod aeromancer;
pub mod paladin;
pub mod gunlancer;
pub mod slayer;
pub mod shadowhunter;
pub mod deadeye;
pub mod summoner;
pub mod artist;
pub mod arcanist;
pub mod artillerist;
pub mod bard;
pub mod scrapper;
pub mod machinist;
pub mod deathblade;
pub mod souleater;
pub mod striker;
pub mod wildsoul;
pub mod builder;

pub use builder::PlayerTemplateBuilder;

use chrono::Duration;

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
    pub buff_duration: Option<Duration>,
    pub cooldown: Duration,
    pub cooldown_gem: f32,
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