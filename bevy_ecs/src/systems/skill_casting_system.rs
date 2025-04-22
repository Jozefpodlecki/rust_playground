use std::{io::{stdout, Write}, time::{Duration, Instant}};

use bevy::{time::Time, utils::HashMap};
use bevy_ecs::prelude::*;
use crossterm::{cursor::MoveTo, style::Print, terminal::{Clear, ClearType}, ExecutableCommand};
use rand::{rng, Rng};

use crate::{components::*, utils::format_number};

pub fn skill_casting_system(
    mut query: Query<(
        &mut Cooldowns,
        &SkillSet,
        &mut AttackTarget,
        &mut DamageMeter,
        &AttackPower,
        &CritRate,
        &mut CastingState
    ), With<Player>>,
    mut target_healths: Query<(Entity, &mut Health), Without<Player>>,
    time: Res<Time>,
    mut commands: Commands,
    raid_clear: Res<RaidClear>,
) {
    if raid_clear.0 {
        return;
    }

    let now = Instant::now();

    for (mut cds, skills, mut target, mut dps, attack_power, crit, mut casting_state) in &mut query {
        if casting_state.is_casting {
            if now.duration_since(casting_state.cast_start_time.unwrap()) >= casting_state.cast_duration {
                casting_state.is_casting = false;
            } else {
                continue;
            }
        }

        for skill in &skills.0 {
            let ready = cds.0.get(&skill.name).map_or(true, |end| *end <= now);
            if ready {
                casting_state.is_casting = true;
                casting_state.cast_duration = skill.casting_duration;
                casting_state.cast_start_time = Some(now);

                cds.0.insert(skill.name.clone(), now + skill.cooldown);

                if skill.is_damage {
                    if let Ok((entity, mut health)) = target_healths.get_mut(target.0) {
                        let (damage, is_crit) = skill.calculate_damage(attack_power.0, crit.0);
                        health.current = health.current.saturating_sub(damage);
                        dps.0 += damage;
                        health.update_health_bars();

                        if health.current == 0 {
                            commands.entity(entity).insert(Dead);
                        }
                    }
                }

                break;
            }
        }
    }
}