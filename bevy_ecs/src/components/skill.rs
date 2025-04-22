use std::time::{Duration, Instant};

use bevy_ecs::prelude::*;
use rand::{rng, Rng};

use super::*;

#[derive(Clone)]
pub struct Skill {
    pub name: String,
    pub priority: u8,
    pub is_damage: bool,
    pub min_ratio: u64,
    pub max_ratio: u64,
    pub cooldown: Duration,
    pub buffs: Vec<Buff>,
    pub casting_duration: Duration,
}

impl Skill {
    pub fn calculate_damage(&self, attack_power: u64, crit_rate: f64) -> (u64, bool){
        let mut rng = rng();

        let is_crit = rng.random_bool(crit_rate);
        let mut damage = attack_power * rng.random_range(self.min_ratio..self.max_ratio);
        let damage = if is_crit {
            (damage as f32 * 1.5) as u64
        } else {
            damage
        };

        (damage, is_crit)
    }
}

#[derive(Component)]
pub struct SkillSet(pub Vec<Skill>);

impl SkillSet {
    pub fn bard() -> Self {
        Self(vec![
            Skill {
                name: "Brand".to_string(),
                priority: 1,
                is_damage: true,
                min_ratio: 100,
                max_ratio: 200,
                cooldown: Duration::from_secs(5),
                buffs: vec![Buff::temporary(BuffType::Brand, 0.1, 10.0)],
                casting_duration: Duration::from_millis(250),
            },
            Skill {
                name: "Buff".to_string(),
                priority: 2,
                is_damage: false,
                min_ratio: 0,
                max_ratio: 0,
                cooldown: Duration::from_secs(10),
                buffs: vec![Buff::temporary(BuffType::AttackPower, 0.1, 10.0)],
                casting_duration: Duration::from_millis(250),
            },
        ])
    }

    pub fn berserker() -> Self {
        Self(vec![
            Skill {
                name: "Smash".to_string(),
                priority: 1,
                is_damage: true,
                min_ratio: 1000,
                max_ratio: 2000,
                cooldown: Duration::from_secs(3),
                buffs: vec![],
                casting_duration: Duration::from_millis(250),
            },
            Skill {
                name: "Berserker Rage".to_string(),
                priority: 2,
                is_damage: true,
                min_ratio: 1200,
                max_ratio: 2500,
                cooldown: Duration::from_secs(6),
                buffs: vec![],
                casting_duration: Duration::from_millis(250),
            },
        ])
    }

    pub fn aeromancer() -> Self {
        Self(vec![
            Skill {
                name: "Wind Slash".to_string(),
                priority: 1,
                is_damage: true,
                min_ratio: 500,
                max_ratio: 1500,
                cooldown: Duration::from_secs(4),
                buffs: vec![],
                casting_duration: Duration::from_millis(250),
            },
            Skill {
                name: "Gale Force".to_string(),
                priority: 2,
                is_damage: true,
                min_ratio: 700,
                max_ratio: 1800,
                cooldown: Duration::from_secs(6),
                buffs: vec![],
                casting_duration: Duration::from_millis(250),
            },
        ])
    }

    pub fn sorceress() -> Self {
        Self(vec![
            Skill {
                name: "Fireball".to_string(),
                priority: 1,
                is_damage: true,
                min_ratio: 800,
                max_ratio: 1600,
                cooldown: Duration::from_secs(6),
                buffs: vec![],
                casting_duration: Duration::from_millis(250),
            },
            Skill {
                name: "Flame Wave".to_string(),
                priority: 2,
                is_damage: true,
                min_ratio: 1200,
                max_ratio: 2500,
                cooldown: Duration::from_secs(8),
                buffs: vec![],
                casting_duration: Duration::from_millis(250), 
            },
        ])
    }
}

#[derive(Default, Component)]
pub struct CastingState {
    pub is_casting: bool,
    pub cast_duration: Duration,
    pub cast_start_time: Option<Instant>,
}