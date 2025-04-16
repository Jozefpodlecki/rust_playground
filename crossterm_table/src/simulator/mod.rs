mod create_party_from_templates;
mod apply_buffs;
mod perform_attack;

use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use rand::{Rng, rng};
use uuid::Uuid;

use crate::{models::{player_template::*, *}, utils::*};

pub struct Simulator {
    encounter: Encounter,
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

        let parties = Self::create_party_from_templates(
            &mut player_templates_map,
            &mut player_states,
            &mut party_states,
            &mut player_templates);

        let encounter = Encounter {
            id: Uuid::nil(),
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

        let mut boss_state = BossState::default();
        boss_state.current_hp = encounter.boss.current_hp;
        boss_state.id = encounter.boss.id;

        Self {
            encounter,
            player_templates: player_templates_map,
            player_states,
            party_states,
            boss_state
        }
    }

    pub fn start(&mut self) {
        self.encounter.started_on = Utc::now();
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

    pub fn progress(&mut self) {
        let now = Utc::now();
        let elapsed_duration = now - self.encounter.started_on;
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
            return;
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
                        &mut player.stats,
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
    }

    pub fn get_encounter(&mut self) -> &Encounter {
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
        player_stats: &mut PlayerStats,
        duration_seconds: i64,
        attack_result: &AttackResult) {

        if attack_result.damage == 0 {
            return;
        }

        player_stats.total_damage += attack_result.damage;

        if duration_seconds != 0 {
            player_stats.dps = (player_stats.total_damage as f32 / duration_seconds as f32) as u64;
        }

        player_stats.crit_damage += if attack_result.is_critical {
            attack_result.damage
        } else {
            0
        };
        
        player_stats.crit_rate = player_stats.crit_damage as f32 / player_stats.total_damage as f32;

        if attack_result.with_brand {
            player_stats.damage_with_brand += attack_result.damage;
            player_stats.brand_percentage = player_stats.damage_with_brand as f32 / player_stats.total_damage as f32;
        }

        if attack_result.with_attack_power_buff {
            player_stats.damage_with_attack_power_buff += attack_result.damage;
            player_stats.attack_power_buff_percentage = player_stats.damage_with_attack_power_buff as f32 / player_stats.total_damage as f32;
        }

        if attack_result.with_identity_buff {
            player_stats.damage_with_identity_buff += attack_result.damage;
            player_stats.identity_percentage = player_stats.damage_with_identity_buff as f32 / player_stats.total_damage as f32;
        }

        if attack_result.with_hat_buff {
            player_stats.damage_with_hat_buff += attack_result.damage;
            player_stats.hat_percentage = player_stats.damage_with_hat_buff as f32 / player_stats.total_damage as f32;
        }

        let skill_stats = player_stats.skills.skill.entry(attack_result.skill_id).or_default();

        if attack_result.is_critical {
            player_stats.skills.crit_count += 1;
            skill_stats.crit_count += 1;
        }

        if attack_result.is_hyper_awakening {
            player_stats.hyper_awakening_damage += attack_result.damage;
        }

        if attack_result.hit_option == HitOption::Back {
            skill_stats.back_attack.count += 1;
            skill_stats.back_attack.damage += attack_result.damage;
        }

        if attack_result.hit_option == HitOption::Frontal {
            skill_stats.front_attack.count += 1;
            skill_stats.front_attack.damage += attack_result.damage;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;


}