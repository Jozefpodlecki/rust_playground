use std::{io::{stdout, Write}, time::{Duration, Instant}};

use bevy::{time::Time, utils::HashMap};
use bevy_ecs::prelude::*;
use crossterm::{cursor::MoveTo, style::Print, terminal::{Clear, ClearType}, ExecutableCommand};
use rand::{rng, Rng};

use crate::{components::*, utils::format_number};

pub fn setup(mut commands: Commands) {
    commands.insert_resource(RaidClear(false));
    commands.insert_resource(PhaseIndex(0));

    let mut stdout = stdout();
    let _ = stdout.execute(Clear(ClearType::All));
    commands.insert_resource(StdoutResource(stdout));

    let boss = commands.spawn((
        Boss,
        Name("Behemoth".to_string()),
        Health::new(500_000_000_000, 500),
    )).id();

    for party_id in 0..4 {
        for i in 0..4 {
            let is_support = i == 3;
            let player_id = i;
            let name = format!("Party-{}-Player-{}", party_id + 1, i + 1);

            let mut player = commands.spawn((
                Name(name),
                Player(player_id),
                Party(party_id),
                Health::new(100_000, 1),
                CritRate(0.1),
                Buffs(vec![BuffType::IncreaseHp10, BuffType::IncreaseCrit10]),
                Cooldowns(HashMap::new()),
                DamageMeter(0),
                SkillSet(vec![
                    Skill { name: "Slash".to_string(), damage: 2_00000_000, cooldown: Duration::from_secs(2) },
                    Skill { name: "Heavy Blow".to_string(), damage: 3_00000_000, cooldown: Duration::from_secs(2) },
                ]),
                AttackTarget(boss),
            ));

            if is_support {
                player.insert(Support);
            } else {
                player.insert(DPS);
            }
        }
    }
}


pub fn apply_buffs(mut query: Query<(&mut Health, &mut CritRate, &Buffs), Added<Buffs>>) {
    for (mut hp, mut crit, buffs) in &mut query {
        for buff in &buffs.0 {
            match buff {
                BuffType::IncreaseHp10 => {
                    hp.max = (hp.max as f32 * 1.10) as u64;
                    hp.current = hp.max;
                }
                BuffType::IncreaseCrit10 => {
                    crit.0 += 0.10;
                }
            }
        }
    }
}

pub fn skill_casting_system(
    mut query: Query<(&mut Cooldowns, &SkillSet, &mut AttackTarget, &mut DamageMeter, &CritRate), With<Player>>,
    mut target_healths: Query<(Entity, &mut Health), Without<Player>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let now = Instant::now();
    for (mut cds, skills, mut target, mut dps, crit) in &mut query {
        if let Ok((entity, mut health)) = target_healths.get_mut(target.0) {
            for skill in &skills.0 {
                let ready = cds.0.get(&skill.name).map_or(true, |end| *end <= now);
                if ready {
                    let mut rng = rng();
                    let is_crit = rng.random_bool(crit.0);
                    let damage = if is_crit {
                        (skill.damage as f32 * 1.5) as u64
                    } else {
                        skill.damage
                    };

                    health.current = health.current.saturating_sub(damage);
                    dps.0 += damage;
                    cds.0.insert(skill.name.clone(), now + skill.cooldown);
                    health.update_health_bars();

                    if health.current == 0 {
                        commands.entity(entity).insert(Dead);
                    }

                    break;
                }
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
                Health {
                    current: 10_000_000,
                    max: 10_000_000,
                    hp_percentage: 100.0,
                    hp_max_bars: 1,
                    hp_current_bars: 1,
                },
            ));

            phase.0 += 1;
        }
    }
}

pub fn reassign_targets_system(
    minions: Query<Entity, With<Minion>>,
    mut players: Query<(&Party, &mut AttackTarget), With<Player>>,
) {
    for (party, mut target) in &mut players {
        if let Some(minion_entity) = minions.iter().nth(party.0) {
            target.0 = minion_entity;
        }
    }
}


pub fn print_encounter(
    boss_query: Query<(&Name, &Health), With<Boss>>,
    player_query: Query<(&Name, &Party, &DamageMeter, &AttackTarget), With<Player>>,
    target_names: Query<&Name, Without<Player>>,
    boss_alive_check: Query<(), (With<Boss>, Without<Dead>)>,
    minion_query: Query<(&Name, &Health), With<Minion>>, // Add minion query
    mut stdout_res: ResMut<StdoutResource>,
    mut clear: ResMut<RaidClear>,
) {
    let stdout = &mut stdout_res.0;

    if let Ok((name, health)) = boss_query.get_single() {
        let percentage = (health.current as f64 / health.max as f64) * 100.0;

        let _ = stdout
            .execute(MoveTo(0, 0))
            .and_then(|s| s.execute(Clear(ClearType::CurrentLine)))
            .and_then(|s| {
                s.execute(Print(format!(
                    "🧟 {} HP: {}/{} ({:.1}%)",
                    name.0,
                    format_number(health.current),
                    format_number(health.max),
                    percentage
                )))
            });
    }

    if boss_alive_check.is_empty() && !clear.0 {
        clear.0 = true;
        let _ = stdout
            .execute(MoveTo(0, 1))
            .and_then(|s| s.execute(Clear(ClearType::CurrentLine)))
            .and_then(|s| s.execute(Print("✅ Encounter Cleared!")));
    }

    let mut party_map: HashMap<u8, Vec<(String, u64)>> = HashMap::new();
    let mut party_targets: HashMap<u8, String> = HashMap::new();

    for (name, party, dmg, target) in &player_query {
        let party_id = party.0 as u8;

        party_map.entry(party_id).or_default().push((name.0.clone(), dmg.0));

        if !party_targets.contains_key(&party_id) {
            if let Ok(target_name) = target_names.get(target.0) {
                party_targets.insert(party_id, target_name.0.clone());
            }
        }
    }

    let mut line_index = 3;

    for party_id in 0..4u8 {
        if let Some(mut members) = party_map.get_mut(&party_id) {
            members.sort_by_key(|(_, dmg)| std::cmp::Reverse(*dmg));

            let target = party_targets
                .get(&party_id)
                .map(|s| s.as_str())
                .unwrap_or("Unknown");

            let _ = stdout
                .execute(MoveTo(0, line_index))
                .and_then(|s| s.execute(Clear(ClearType::CurrentLine)))
                .and_then(|s| s.execute(Print(format!(
                    "--- Party {} (Fighting: {}) ---",
                    party_id + 1,
                    target
                ))));
            line_index += 1;

            for (name, dmg) in members {
                let _ = stdout
                    .execute(MoveTo(0, line_index))
                    .and_then(|s| s.execute(Clear(ClearType::CurrentLine)))
                    .and_then(|s| s.execute(Print(format!(
                        "{} - 🗡️ Damage: {}",
                        name,
                        format_number(*dmg)
                    ))));
                line_index += 1;
            }
        }
    }

    // Display Minion health percentage
    for (name, health) in &minion_query {
        let _ = stdout
            .execute(MoveTo(0, line_index))
            .and_then(|s| s.execute(Clear(ClearType::CurrentLine)))
            .and_then(|s| s.execute(Print(format!(
                "Minion {} - HP: {:.1}%",
                name.0,
                health.hp_percentage
            ))));
        line_index += 1;
    }

    let _ = stdout.flush();
}

