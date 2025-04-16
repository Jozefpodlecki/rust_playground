use crate::models::PlayerTemplate;

use super::*;

impl Simulator {
    pub fn perform_attack(
        now: DateTime<Utc>,
        boss_state: &mut BossState,
        party_state: &mut PartyState,
        player_state: &mut PlayerState,
        duration_seconds: i64,
        player_template: &PlayerTemplate
    ) -> Option<AttackResult> {
        let mut rng = rng();

        let sorted_skills = Self::get_available_skills(now, player_state, player_template);

        for skill_template in sorted_skills {
            if skill_template.kind == SkillType::Awakening && duration_seconds < 180 {
                continue;
            }

            if skill_template.requires_identity && player_state.identity < 1.0 {
                continue;
            }

            let expires_on = now + skill_template.cooldown;
            player_state
                .skill_cooldowns
                .insert(skill_template.id, expires_on);

            player_state.identity += skill_template.identity_gain;

            let mut result = AttackResult::default();
            result.skill_id = skill_template.id;
            let buff_attack_power = (player_template.attack_power as f32  * 0.15) as u64;

            Self::apply_buffs(
                buff_attack_power,
                &skill_template.buffs,
                now,
                party_state,
                boss_state,
                player_state);
          
            let mut attack_power = player_template.attack_power;
            let mut damage_multiplier = 1.0;

            for (_, buff) in &boss_state.active_debuffs {
                if buff.kind == BuffType::Brand {
                    result.with_brand = true;
                    damage_multiplier += 0.1;
                }
            }

            for (_, buff) in &party_state.active_buffs {
                if buff.expires_on > now {
                    match buff.kind {
                        BuffType::AttackPowerBuff => {
                            result.with_attack_power_buff = true;
                            attack_power += buff.value as u64;
                        },
                        BuffType::Identity => {
                            result.with_identity_buff = true;
                            damage_multiplier += 0.1;
                        },
                        BuffType::HyperAwakeningTechnique => {
                            result.with_hat_buff = true;
                            damage_multiplier += 0.1;
                        },
                        _ => {}
                    }
                }
            }

            let mut damage = 0f32;

            if skill_template.min_ratio != 0.0 {
                let min = attack_power as f32 * skill_template.min_ratio * damage_multiplier;
                let max = attack_power as f32 * skill_template.max_ratio * damage_multiplier;
                damage = rng.random_range(min..max);
            }

            let is_critical = rng.random_bool(player_template.crit_rate as f64);

            result.damage = if is_critical { (damage * player_template.crit_damage) as u64 } else { damage as u64 };
            result.is_critical;

            result.target_id = boss_state.id;

            return Some(result);
        }

        None
    }
}