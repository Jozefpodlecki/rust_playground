
use bevy::utils::HashMap;
use bevy_ecs::prelude::*;
use std::collections::HashSet;
use std::{io::Stdout, time::{Duration, Instant}};

#[derive(Component)]
pub struct Buffs(pub Vec<Buff>);

#[derive(Clone)]
pub struct Buff {
    pub kind: BuffType,
    pub modifier: f64,
    pub expiry: Option<Instant>,
}

#[derive(Clone)]
pub enum BuffType {
    IncreaseHp,
    IncreaseSwift,
    IncreaseCrit,
    Brand,
    AttackPower,
}

impl Buff {
    pub fn permanent(kind: BuffType, modifier: f64) -> Self {
        Buff {
            kind,
            modifier,
            expiry: None,
        }
    }

    pub fn temporary(kind: BuffType, modifier: f64, duration: f64) -> Self {
        Buff {
            kind,
            modifier,
            expiry: Some(Instant::now() + Duration::from_secs_f64(duration)),
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expiry) = self.expiry {
            Instant::now() > expiry
        } else {
            false
        }
    }
}
