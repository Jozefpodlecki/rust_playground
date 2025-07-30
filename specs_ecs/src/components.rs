use specs::{prelude::*, Component};
use std::collections::HashMap;

#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct Name(pub String);

#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

#[derive(Debug, Clone)]
pub struct Skill {
    pub id: u32,
    pub name: String,
    pub cooldown: i32,
    pub last_used: i32,
    pub effects: Vec<SkillEffect>,
}

#[derive(Debug, Clone)]
pub enum SkillEffect {
    Damage(i32),
    Heal(i32),
    Buff {
        stat: String,
        amount: i32,
        duration: i32,
    },
    Debuff {
        stat: String,
        amount: i32,
        duration: i32,
    },
    Summon(u32),
}


#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub enum Class {
    Warrior,
    Mage,
    Support,
}


#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct IsPlayer;

#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct IsBoss;

#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct Skills {
    pub active: Vec<u32>,
}

#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct Cooldowns {
    pub map: HashMap<u32, i32>,
}

#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct Effects {
    pub skill_id: u32,
    pub heal: Option<i32>,
    pub damage: Option<i32>,
    pub buff: Option<u32>,
    pub debuff: Option<u32>,
    pub summon: bool,
}

#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct Owner(pub Entity);

#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct Target(pub Entity);

#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct IsSupport;