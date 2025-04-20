use std::{io::{stdout, Write}, time::{Duration, Instant}};

use bevy::{time::Time, utils::HashMap};
use bevy_ecs::prelude::*;
use crossterm::{cursor::MoveTo, style::Print, terminal::{Clear, ClearType}, ExecutableCommand};
use rand::{rng, Rng};

use crate::{components::*, utils::format_number};

pub fn skill_casting_system(
    mut query: Query<(&mut Cooldowns, &SkillSet, &mut AttackTarget, &mut DamageMeter, &AttackPower, &CritRate), With<Player>>,
    mut target_healths: Query<(Entity, &mut Health), Without<Player>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let now = Instant::now();
    for (mut cds, skills, mut target, mut dps, attack_power, crit) in &mut query {
        if let Ok((entity, mut health)) = target_healths.get_mut(target.0) {
            for skill in &skills.0 {
                let ready = cds.0.get(&skill.name).map_or(true, |end| *end <= now);
                if ready {
                    
                    if skill.is_damage {
                        let (damage, is_crit) = skill.calculate_damage(attack_power.0, crit.0);
                        health.current = health.current.saturating_sub(damage);
                        dps.0 += damage;
                        health.update_health_bars();
                    }

                    cds.0.insert(skill.name.clone(), now + skill.cooldown);

                    if health.current == 0 {
                        commands.entity(entity).insert(Dead);
                    }

                    break;
                }
            }
        }
    }
}
