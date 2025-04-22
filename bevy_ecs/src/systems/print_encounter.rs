use std::{io::{stdout, Write}, time::{Duration, Instant}};

use bevy::{time::Time, utils::HashMap};
use bevy_ecs::prelude::*;
use crossterm::{cursor::MoveTo, style::Print, terminal::{Clear, ClearType}, ExecutableCommand};
use rand::{rng, Rng};

use crate::{components::*, utils::format_number};

pub fn print_encounter(
    boss_query: Query<(&Name, &Health), With<Boss>>,
    player_query: Query<(&Name, &Class, &Party, &DamageMeter, &AttackTarget, &CritRate), With<Player>>,
    target_names: Query<&Name, Without<Player>>,
    boss_alive_check: Query<(), (With<Boss>, Without<Dead>)>,
    minion_query: Query<(&Name, &Health), With<Minion>>,
    mut stdout_res: ResMut<StdoutResource>,
    mut clear: ResMut<RaidClear>,
) {
    let stdout = &mut stdout_res.0;

    if let Ok((name, health)) = boss_query.get_single() {
        let _ = stdout
            .execute(MoveTo(0, 0))
            .and_then(|s| s.execute(Clear(ClearType::CurrentLine)))
            .and_then(|s| {
                s.execute(Print(format!(
                    "{} HP: {}/{} ({:.1}%)",
                    name.0,
                    format_number(health.current),
                    format_number(health.max),
                    health.hp_percentage
                )))
            });
    }

    if boss_alive_check.is_empty() && !clear.0 {
        clear.0 = true;
        let _ = stdout
            .execute(MoveTo(0, 1))
            .and_then(|s| s.execute(Clear(ClearType::CurrentLine)))
            .and_then(|s| s.execute(Print("Encounter Cleared!")));
    }

    let mut party_map: HashMap<u8, Vec<(String, Class, u64, f64)>> = HashMap::new();
    let mut party_targets: HashMap<u8, String> = HashMap::new();

    for (name, class, party, dmg, target, crit) in &player_query {
        let party_id = party.0 as u8;

        party_map
            .entry(party_id)
            .or_default()
            .push((name.0.clone(), *class, dmg.0, crit.0));

        if !party_targets.contains_key(&party_id) {
            if let Ok(target_name) = target_names.get(target.0) {
                party_targets.insert(party_id, target_name.0.clone());
            }
        }
    }

    let mut line_index = 3;

    for party_id in 0..4u8 {
        if let Some(mut members) = party_map.get_mut(&party_id) {
            members.sort_by_key(|(_, _, dmg, _)| std::cmp::Reverse(*dmg));

            if let Some((minion_name, minion_health)) = minion_query.iter().nth(party_id as usize) {
                let _ = stdout
                    .execute(MoveTo(0, line_index))
                    .and_then(|s| s.execute(Clear(ClearType::CurrentLine)))
                    .and_then(|s| s.execute(Print(format!(
                        "--- Party {} (Minion HP: {:.1}%) ---",
                        party_id + 1,
                        minion_health.hp_percentage
                    ))));
            } else {
                if let Some(target_name) = party_targets.get(&party_id) {
                    let _ = stdout
                        .execute(MoveTo(0, line_index))
                        .and_then(|s| s.execute(Clear(ClearType::CurrentLine)))
                        .and_then(|s| s.execute(Print(format!(
                            "--- Party {} (Target: {}) ---",
                            party_id + 1,
                            target_name
                        ))));
                }
            }

            line_index += 1;

            for (name, class, dmg, crit) in members {
                let _ = stdout
                    .execute(MoveTo(0, line_index))
                    .and_then(|s| s.execute(Clear(ClearType::CurrentLine)))
                    .and_then(|s| s.execute(Print(format!(
                        "{} {} - Damage: {} | Crit: {:.1}%",
                        name,
                        class.as_ref(),
                        format_number(*dmg),
                        *crit * 100.0
                    ))));
                line_index += 1;
            }
        }
    }

    let _ = stdout.flush();
}