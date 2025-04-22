use bevy_ecs::prelude::*;
use strum_macros::{AsRefStr, EnumString};

use super::*;
#[derive(Component)]
pub struct Player(pub i32);

#[derive(Bundle)]
pub struct PlayerBundle {
    pub name: Name,
    pub player: Player,
    pub health: Health,
    pub attack_power: AttackPower,
    pub crit_rate: CritRate,
    pub swiftness: Swiftness,
    pub buffs: Buffs,
    pub cooldowns: Cooldowns,
    pub damage_meter: DamageMeter,
    pub class: Class,
    pub skill_set: SkillSet,
    pub attack_target: AttackTarget,
    pub casting_state: CastingState,
}


#[derive(Component, Default, Debug, Copy, Clone, AsRefStr, PartialEq, EnumString)]
pub enum Class {
    #[default]
    Berserker,
    Bard,
    Aeromancer,
    Sorceress
}

#[derive(Component)]
pub struct Support;

#[derive(Component)]
pub struct DamageDealer;

#[derive(Component)]
pub struct AttackPower(pub u64);

#[derive(Component)]
pub struct Swiftness(pub u64);

#[derive(Component)]
pub struct CritRate(pub f64);
