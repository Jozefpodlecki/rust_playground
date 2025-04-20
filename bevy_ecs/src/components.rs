pub use std::collections::HashSet;
use std::{io::Stdout, time::{Duration, Instant}};

use bevy::utils::HashMap;
pub use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Player(pub i32);

#[derive(Component)]
pub struct Support;

#[derive(Component)]
pub struct DPS;

#[derive(Component)]
pub struct Boss;

#[derive(Component)]
pub struct Minion;

#[derive(Component)]
pub struct Party(pub usize);

#[derive(Component)]
pub struct Name(pub String);

#[derive(Component)]
pub struct Health {
    pub current: u64,
    pub max: u64,
    pub hp_percentage: f64,
    pub hp_max_bars: u64,
    pub hp_current_bars: u64,
}

impl Health {
    pub fn new(max_hp: u64, max_bars: u64) -> Self {
        let current = max_hp;
        let hp_percentage = 100.0;
        let hp_current_bars = max_bars;

        Health {
            current,
            max: max_hp,
            hp_percentage,
            hp_max_bars: max_bars,
            hp_current_bars,
        }
    }

    pub fn update_health_bars(&mut self) {
        self.hp_percentage = (self.current as f64 / self.max as f64) * 100.0;
        self.hp_current_bars = ((self.hp_percentage / 100.0) * self.hp_max_bars as f64).round() as u64;
    }
}

#[derive(Component)]
pub struct Buffs(pub Vec<BuffType>);

#[derive(Component)]
pub struct AttackTarget(pub Entity);

#[derive(Component)]
pub struct CritRate(pub f64);

#[derive(Component)]
pub struct Dead;

#[derive(Resource)]
pub struct RaidClear(pub bool);

#[derive(Resource)]
pub struct PhaseIndex(pub usize);

#[derive(Clone)]
pub enum BuffType {
    IncreaseHp10,
    IncreaseCrit10,
}

#[derive(Clone)]
pub struct Skill {
    pub name: String,
    pub damage: u64,
    pub cooldown: Duration,
}

#[derive(Component)]
pub struct SkillSet(pub Vec<Skill>);

#[derive(Component)]
pub struct Cooldowns(pub HashMap<String, Instant>);

#[derive(Resource)]
pub struct StdoutResource(pub Stdout);

#[derive(Component)]
pub struct DamageMeter(pub u64);