mod create_party_from_templates;
mod apply_buffs;

use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use rand::{Rng, rng};

use crate::{models::{player_template::*, *}, utils::*};

#[derive(Clone)]
pub struct Buff {
    pub target: BuffTarget,
    pub kind: BuffType,
    pub expires_on: DateTime<Utc>,
    pub value: u64
}

#[derive(Default)]
pub struct PlayerState {
    pub skill_cooldowns: HashMap<u32, DateTime<Utc>>,
    pub active_buffs: HashMap<u32, Buff>,
    pub identity: f32
}

#[derive(Default)]
pub struct BossState {
    pub active_debuffs: HashMap<u32, Buff>,
}

pub struct PartyState {
    pub active_buffs: HashMap<u32, Buff>,
}

pub struct Simulator {
    encounter: Encounter,
    started_on: DateTime<Utc>,
    player_templates: HashMap<u64, PlayerTemplate>,
    player_states: HashMap<u64, PlayerState>,
    party_states: HashMap<u64, PartyState>,
    boss_state: BossState,
}

impl Simulator {
    pub fn new(
        encounter_template: EncounterTemplate,
        mut player_templates: Vec<PlayerTemplate>,
    ) -> Self {

        let started_on = Utc::now();
        let mut player_templates_map: HashMap<u64, PlayerTemplate> = HashMap::new();
        let mut player_states: HashMap<u64, PlayerState> = HashMap::new();
        let mut party_states : HashMap<u64, PartyState> = HashMap::new();
        let boss_state = BossState::default();

        let parties = Self::create_party_from_templates(
            &mut player_templates_map,
            &mut player_states,
            &mut party_states,
            &mut player_templates);

        let encounter = Encounter {
            boss: Boss { 
                id: random_number_in_range(1000..9999),
                name: encounter_template.boss.name,
                max_hp: encounter_template.boss.max_hp,
                current_hp: encounter_template.boss.max_hp,
                hp_percentage: 1.0,
                hp_bars: encounter_template.boss.hp_bars
            },
            duration: EncounterDuration {
                elapsed_seconds: 0,
                mmss: "00:00".to_string(),
            },
            started_on,
            parties,
            stats: EncounterStats { 
                total_damage: 0,
                ttk: "INF".to_string(),
                dps: 0
            }
        };

        Self {
            encounter,
            started_on,
            player_templates: player_templates_map,
            player_states,
            party_states,
            boss_state
        }
    }

    pub fn start(&mut self) {
        self.started_on = Utc::now();
    }

    fn get_available_skills<'a>(
        now: DateTime<Utc>,
        player_state: &mut PlayerState,
        player_template: &'a PlayerTemplate) -> Vec<&'a SkillTemplate> {
        let available_skills: Vec<_> = player_template
            .skills
            .iter()
            .filter(|skill| {
                let cooldown = player_state
                    .skill_cooldowns
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
                            attack_power += buff.value;
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

            return Some(result);
        }

        None
    }

    pub fn tick(&mut self) -> &Encounter {
        let now = Utc::now();
        let elapsed_duration = now - self.started_on;
        let duration_seconds = elapsed_duration.num_seconds();
        let formatted_duration = format_duration(duration_seconds);
        let encounter = &mut self.encounter;
        let parties = &mut encounter.parties;
        let boss_state = &mut self.boss_state;
        boss_state.active_debuffs.retain(|_, buff| buff.expires_on > now);

        encounter.duration = EncounterDuration {
            elapsed_seconds: duration_seconds as u64,
            mmss: formatted_duration,
        };

        if encounter.boss.current_hp == 0 {
            return &self.encounter;
        }

        for party in parties {

            let party_state = self.party_states.get_mut(&party.id).unwrap();
            party_state.active_buffs.retain(|_, buff| buff.expires_on > now);

            for player in party.players.iter_mut() {
                let player_state = self.player_states.get_mut(&player.id).unwrap();                
                player_state.skill_cooldowns.retain(|_, cooldown| *cooldown > now);
                player_state.active_buffs.retain(|_, buff| buff.expires_on > now);

                let player_template = self.player_templates.get(&player.id).unwrap();
                
                if let Some(attack_result) =
                    Self::perform_attack(
                        now,
                        boss_state,
                        party_state,
                        player_state,
                        duration_seconds,
                        player_template)
                {
                    Self::update_player_stats(
                        player,
                        duration_seconds,
                        &attack_result);

                    Self::update_encounter_stats(
                        duration_seconds,
                        &mut encounter.boss,
                        &mut encounter.stats,
                        &attack_result);
                    Self::update_party_stats(
                        duration_seconds,
                        &mut party.stats,
                        &attack_result,
                        encounter.stats.total_damage);
                }
            }
        }

        for party in &mut encounter.parties {
            party.players.sort_by(|a, b| b.stats.dps.cmp(&a.stats.dps));
        }

        &self.encounter
    }

    fn update_party_stats(
        duration_seconds: i64,
        party_stats: &mut PartyStats,
        attack_result: &AttackResult,
        total_damage: u64,
    ) {
        if attack_result.damage == 0 {
            return;
        }

        party_stats.total_damage += attack_result.damage;

        if duration_seconds == 0 {
            party_stats.dps = 0;
        }
        else {
            party_stats.dps = party_stats.total_damage / duration_seconds as u64;
            party_stats.total_damage_percentage = party_stats.total_damage as f32 / total_damage as f32;
        }
    }

    fn update_encounter_stats(
        duration_seconds: i64,
        boss: &mut Boss,
        encounter_stats: &mut EncounterStats,
        attack_result: &AttackResult) {

        if attack_result.damage == 0 {
            return;
        }

        if attack_result.damage >= boss.current_hp {
            boss.current_hp = 0;
            let damage = attack_result.damage - boss.current_hp;
            encounter_stats.total_damage += damage;
            encounter_stats.ttk = "00:00".to_string();
        }
        else {
            boss.current_hp = boss.current_hp - attack_result.damage;
            boss.hp_percentage = boss.current_hp as f32 / boss.max_hp as f32;
            encounter_stats.total_damage += attack_result.damage;
            
            if duration_seconds == 0 {
                encounter_stats.dps = 0;
                encounter_stats.ttk = "INF".to_string();
            }
            else {
                encounter_stats.dps = encounter_stats.total_damage / duration_seconds as u64;   
                let ttk_seconds = boss.current_hp / encounter_stats.dps;
                encounter_stats.ttk = format_duration(ttk_seconds as i64);
            }
        }
    }

    fn update_player_stats(
        player: &mut Player,
        duration_seconds: i64,
        attack_result: &AttackResult) {

        if attack_result.damage == 0 {
            return;
        }

        player.stats.total_damage += attack_result.damage;

        if duration_seconds != 0 {
            player.stats.dps = (player.stats.total_damage as f32 / duration_seconds as f32) as u64;
        }

        player.stats.crit_damage += if attack_result.is_critical {
            attack_result.damage
        } else {
            0
        };
        
        player.stats.crit_rate = player.stats.crit_damage as f32 / player.stats.total_damage as f32;

        if attack_result.with_brand {
            player.stats.damage_with_brand += attack_result.damage;
            player.stats.brand_percentage = player.stats.damage_with_brand as f32 / player.stats.total_damage as f32;
        }

        if attack_result.with_attack_power_buff {
            player.stats.damage_with_attack_power_buff += attack_result.damage;
            player.stats.attack_power_buff_percentage = player.stats.damage_with_attack_power_buff as f32 / player.stats.total_damage as f32;
        }

        if attack_result.with_identity_buff {
            player.stats.damage_with_identity_buff += attack_result.damage;
            player.stats.identity_percentage = player.stats.damage_with_identity_buff as f32 / player.stats.total_damage as f32;
        }

        if attack_result.with_hat_buff {
            player.stats.damage_with_hat_buff += attack_result.damage;
            player.stats.hat_percentage = player.stats.damage_with_hat_buff as f32 / player.stats.total_damage as f32;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;


}