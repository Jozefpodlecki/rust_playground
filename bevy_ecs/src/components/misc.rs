
use std::collections::HashSet;
use std::{io::Stdout, time::{Duration, Instant}};

use bevy::utils::HashMap;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Boss;

#[derive(Component)]
pub struct Minion;

#[derive(Component)]
pub struct Party(pub usize);

#[derive(Component)]
pub struct Name(pub String);

#[derive(Component)]
pub struct AttackTarget(pub Entity);

#[derive(Component)]
pub struct Dead;

#[derive(Resource)]
pub struct RaidClear(pub bool);

#[derive(Resource)]
pub struct PhaseIndex(pub usize);

#[derive(Component)]
pub struct Cooldowns(pub HashMap<String, Instant>);

#[derive(Resource)]
pub struct StdoutResource(pub Stdout);

#[derive(Component)]
pub struct DamageMeter(pub u64);