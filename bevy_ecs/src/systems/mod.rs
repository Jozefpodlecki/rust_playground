use std::{io::{stdout, Write}, time::{Duration, Instant}};

use bevy::{time::Time, utils::HashMap};
use bevy_ecs::prelude::*;
use crossterm::{cursor::MoveTo, style::Print, terminal::{Clear, ClearType}, ExecutableCommand};
use rand::{rng, Rng};

use crate::{components::*, utils::format_number};

pub mod print_encounter;
pub mod setup;
pub mod skill_casting_system;

pub use skill_casting_system::*;
pub use print_encounter::*;
pub use setup::*;

pub fn apply_initial_buffs(mut query: Query<(&mut Health, &mut Swiftness, &mut CritRate, &Buffs), Added<Buffs>>) {
    for (mut hp, mut swiftness, mut crit, buffs) in query.iter_mut() {
        
        for buff in buffs.0.clone() {
            match buff.kind {
                BuffType::IncreaseHp => {
                    hp.max = (hp.max as f64 * (1.0 + buff.modifier)) as u64;
                    hp.current = hp.max;
                }
                BuffType::IncreaseCrit => {
                    crit.0 += buff.modifier;
                }
                BuffType::IncreaseSwift => {
                    swiftness.0 = (swiftness.0 as f64 * (1.0 + buff.modifier as f64)) as u64;
                },
                _ => {}
            }
        }
    }
}

pub fn phase_trigger_system(
    mut commands: Commands,
    mut phase: ResMut<PhaseIndex>,
    boss_query: Query<(&Health, Entity), (With<Boss>, Without<Dead>)>,
) {
    if let Ok((health, _)) = boss_query.get_single() {
        let bar = (health.current as f32 / health.max as f32) * 500.0;
        let triggers = [420.0, 340.0, 260.0, 180.0];

        if phase.0 < triggers.len() && bar <= triggers[phase.0] {
            let minion_name = format!("Minion{}", phase.0 + 1);
            commands.spawn((
                Minion,
                Name(minion_name),
                Health::new(5_000_000_000, 1),
            ));

            phase.0 += 1;
        }
    }
}

pub fn reassign_targets_system(
    minions: Query<(Entity, &Health), With<Minion>>,
    mut players: Query<(&Party, &mut AttackTarget), With<Player>>,
    boss_query: Query<Entity, With<Boss>>,
) {
    for (party, mut target) in &mut players {
        let mut assigned_minion = false;
        for (minion_entity, minion_health) in &minions {
            if minion_health.current > 0 {
                if let Some(minion) = minions.iter().nth(party.0) {
                    target.0 = minion_entity;
                    assigned_minion = true;
                    break;
                }
            }
        }

        if !assigned_minion {
            if let Some(boss) = boss_query.iter().next() {
                target.0 = boss;
            }
        }
    }
}