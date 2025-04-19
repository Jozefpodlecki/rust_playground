use chrono::{DateTime, Utc};
use rand::{Rng, rng};

use crate::models::{player_template::*, *};

use super::*;
use crate::multi_thread_simulator::apply_buffs::apply_buffs;

pub fn get_available_skills<'a>(
    now: DateTime<Utc>,
    skills: &'a [SkillTemplate],
    skill_cooldowns: &HashMap<u32, DateTime<Utc>>) -> Vec<&'a SkillTemplate> {
    let available_skills: Vec<_> = skills.iter()
        .filter(|skill| {
            let cooldown = skill_cooldowns
                .get(&skill.id)
                .cloned()
                .unwrap_or(now);
            cooldown <= now
        })
        .collect();

    let mut sorted_skills = available_skills.clone();
    sorted_skills.sort_by_key(|skill| skill.priority);

    sorted_skills
}

// pub fn perform_attack(
//     id_generator: &mut IdGenerator,
//     player_id: u64,
//     now: DateTime<Utc>,
//     boss_state: Arc<RwLock<BossState>>,
//     party_state: Arc<RwLock<PartyState>>,
//     player_state: &mut PlayerState,
//     duration_seconds: i64,
//     player_template: &PlayerTemplate
// ) -> Option<AttackResult> {
//     let mut rng = rng();

//     let sorted_skills = get_available_skills(
//         now,
//         &player_template.skills,
//         &player_state.skill_cooldowns);

//     for skill_template in sorted_skills {
//         if skill_template.kind == SkillType::Awakening && duration_seconds < 180 {
//             continue;
//         }

//         if skill_template.requires_identity && player_state.identity < 2.0 {
//             continue;
//         }

//         let expires_on = now + skill_template.cooldown;
//         player_state
//             .skill_cooldowns
//             .insert(skill_template.id, expires_on);

//         sleep(skill_template.cast_duration.to_std().unwrap());

//         player_state.identity += skill_template.identity_gain;

//         let mut result = AttackResult::default();
//         result.source_id = player_id;
//         result.skill_id = skill_template.id;
       
//         apply_buffs(
//             id_generator,
//             player_template,
//             &skill_template.buffs,
//             now,
//             party_state.clone(),
//             boss_state.clone(),
//             &mut player_state.active_buffs,
//             &mut HashMap::new());

//         let mut attack_power = player_template.attack_power;
//         let mut damage_multiplier = 1.0;

//         let active_debuffs = &boss_state.read().unwrap().active_debuffs;
//         for (_, buff) in active_debuffs.iter() {
//             if buff.kind == BuffType::Brand {
//                 result.with_brand = true;
//                 damage_multiplier += 0.1;
//             }
//         }

//         for (_, buff) in &party_state.read().unwrap().active_buffs {
//             if buff.expires_on > now {
//                 match buff.kind {
//                     BuffType::AttackPowerBuff => {
//                         result.with_attack_power_buff = true;
//                         attack_power += buff.value as u64;
//                     },
//                     BuffType::Identity => {
//                         result.with_identity_buff = true;
//                         damage_multiplier += 0.1;
//                     },
//                     BuffType::DamageAmplification => {
//                         damage_multiplier += buff.value;
//                     },
//                     BuffType::HyperAwakeningTechnique => {
//                         result.with_hat_buff = true;
//                         damage_multiplier += 0.1;
//                     },
//                     _ => {}
//                 }
//             }
//         }

//         let mut damage = 0f32;

//         if skill_template.min_ratio != 0.0 {
//             let min = attack_power as f32 * skill_template.min_ratio * damage_multiplier;
//             let max = attack_power as f32 * skill_template.max_ratio * damage_multiplier;
//             damage = rng.random_range(min..max);
//         }

//         let is_critical = rng.random_bool(player_template.crit_rate as f64);

//         result.damage = if is_critical { (damage * player_template.crit_damage) as u64 } else { damage as u64 };
//         result.is_critical = is_critical;

//         return Some(result);
//     }

//     None
// }