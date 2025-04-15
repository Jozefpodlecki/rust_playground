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
}

#[derive(Default, Debug, Clone)]
pub struct SkillTemplate {
    pub id: u32,
    pub name: &'static str,
    pub ratio: f32,
    pub kind: SkillType,
    pub priority: u8,
    pub cast_duration: Duration,
    pub buff_duration: Option<Duration>,
    pub cooldown: Duration,
}

#[derive(Default, Debug, Clone)]
pub struct PlayerTemplate {
    pub class: Class,
    pub crit_rate: f32,
    pub cooldown_reduction: f32,
    pub attack_power: u64,
    pub skills: Vec<SkillTemplate>,
}