use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use rand::{Rng, rng};

use crate::{models::{player_template::{SkillTemplate, SkillType}, *}, utils::*};

pub struct Buff {
    pub kind: SkillType,
    pub expires_on: DateTime<Utc>,
    pub value: u64
}

#[derive(Default)]
pub struct PlayerState {
    pub skill_cooldowns: HashMap<u32, DateTime<Utc>>,
    pub active_buffs: HashMap<u32, Buff>,
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
}

impl Simulator {
    pub fn new(
        encounter_template: EncounterTemplate,
        mut player_templates: Vec<PlayerTemplate>,
    ) -> Self {
        let mut player_templates_map: HashMap<u64, PlayerTemplate> = HashMap::new();
        let mut player_states: HashMap<u64, PlayerState> = HashMap::new();
        let mut party_states : HashMap<u64, PartyState> = HashMap::new();
        let started_on = Utc::now();
        let mut parties = Vec::new();

        for chunk in player_templates.chunks_mut(4) {
            let members = Vec::new();
            let mut party= Party {
                id: random_number_in_range(1000..9999),
                players: members,
            };

            for template in chunk {
                let mut id = random_number_in_range(1000..9999);

                while player_templates_map.contains_key(&id) {
                    id = random_number_in_range(1000..9999);
                }

                for skill in template.skills.iter_mut() {
                    let reduction_factor = 1.0 - (template.cooldown_reduction as f32 / 100.0);
                    let reduced_cooldown = skill.cooldown.num_nanoseconds().unwrap() as f32 * reduction_factor;
                    let reduced_cooldown = Duration::nanoseconds(reduced_cooldown as i64);
                    // println!("{}: skill {} cdr {:?} -> {:?}", template.class.as_ref(), skill.id, skill.cooldown, reduced_cooldown);
                    skill.cooldown = reduced_cooldown;
                }

                player_templates_map.insert(id, template.clone());

                player_states.insert(
                    id,
                    PlayerState {
                        skill_cooldowns: template
                            .skills
                            .iter()
                            .map(|skill| (skill.id, Utc::now()))
                            .collect(),
                        active_buffs: HashMap::new(),
                    },
                );

                let player = Player {
                    id,
                    name: random_alphabetic_string_capitalized(8),
                    class: template.class.clone(),
                    stats: PlayerStats {
                        skills: PlayerSkillStats {
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    skills: HashMap::new(),
                };

                party.players.push(player);
            }

            party_states.insert(
                party.id,
                PartyState {
                    active_buffs: HashMap::new(),
                },
            );

            parties.push(party);
        }

        let encounter = Encounter {
            boss: encounter_template.boss,
            duration: EncounterDuration {
                elapsed_seconds: 0,
                mmss: "00:00".to_string(),
            },
            started_on,
            parties,
            stats: EncounterStats { total_damage: 0 }
        };

        Self {
            encounter,
            started_on,
            player_templates: player_templates_map,
            player_states,
            party_states
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

            let expires_on = now + skill_template.cooldown;
            player_state
                .skill_cooldowns
                .insert(skill_template.id, expires_on);

            if matches!(
                skill_template.kind,
                SkillType::Brand | 
                SkillType::AttackPowerBuff | 
                SkillType::Identity | 
                SkillType::HyperAwakeningTechnique
            ) {
                let expires_on = now + skill_template.buff_duration.expect(&format!("should have duration {}", player_template.class.as_ref()));
                let buff = Buff {
                    kind: skill_template.kind,
                    expires_on,
                    value: 0
                };
                party_state.active_buffs.insert(
                    skill_template.id,
                    buff
                );
            }

            let mut attack_power = player_template.attack_power;
            let mut damage_multiplier = 1.0;

            for (_, buff) in &party_state.active_buffs {
                if buff.expires_on > now {
                    match buff.kind {
                        SkillType::Brand => {
                            damage_multiplier *= 1.1;
                        }
                        SkillType::AttackPowerBuff => {
                            attack_power = attack_power + buff.value
                        }
                        SkillType::Identity => {
                            damage_multiplier *= 1.1;
                        }
                        SkillType::HyperAwakeningTechnique => {
                            damage_multiplier *= 1.1;
                        }
                        _ => {}
                    }
                }
            }

            let min = player_template.attack_power as f32 * 0.8 * damage_multiplier;
            let max = player_template.attack_power as f32 * 1.2 * damage_multiplier;
            let damage = rng.random_range(min..max);

            let is_critical = rng.random_bool(player_template.crit_rate as f64);

            let result = AttackResult {
                skill_id: skill_template.id,
                damage: if is_critical { (damage * 2.0) as u64 } else { damage as u64 },
                is_critical,
            };

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

        encounter.duration = EncounterDuration {
            elapsed_seconds: duration_seconds as u64,
            mmss: formatted_duration,
        };

        if encounter.boss.current_hp == 0 {
            return &self.encounter;
        }

        for party in &mut encounter.parties {
            for player in party.players.iter_mut() {
                let player_state = self.player_states.get_mut(&player.id).unwrap();
                let party_state = self.party_states.get_mut(&party.id).unwrap();

                party_state.active_buffs.retain(|_, buff| buff.1 > now);
                player_state.skill_cooldowns.retain(|_, cooldown| *cooldown > now);
                player_state.active_buffs.retain(|_, buff| buff.1 > now);

                let player_template = self.player_templates.get(&player.id).unwrap();
                
                if let Some(attack_result) =
                    Self::perform_attack(
                        now,
                        party_state,
                        player_state,
                        duration_seconds,
                        player_template)
                {
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

                    if party_state.active_buffs.iter()
                        .any(|pr| pr.1.0 == SkillType::Brand) {
                        player.stats.damage_with_brand += attack_result.damage;
                        player.stats.brand_percentage = player.stats.damage_with_brand as f32 / player.stats.total_damage as f32;
                    }

                    if party_state.active_buffs.iter()
                        .any(|pr| pr.1.0 == SkillType::AttackPowerBuff) {
                        player.stats.damage_with_attack_power_buff += attack_result.damage;
                        player.stats.attack_power_buff_percentage = player.stats.damage_with_attack_power_buff as f32 / player.stats.total_damage as f32;
                    }

                    if party_state.active_buffs.iter()
                        .any(|pr| pr.1.0 == SkillType::Identity) {
                        player.stats.damage_with_identity_buff += attack_result.damage;
                        player.stats.identity_percentage = player.stats.damage_with_identity_buff as f32 / player.stats.total_damage as f32;
                    }

                    if party_state.active_buffs.iter()
                        .any(|pr| pr.1.0 == SkillType::HyperAwakeningTechnique) {
                        player.stats.damage_with_hat_buff += attack_result.damage;
                        player.stats.hat_percentage = player.stats.damage_with_hat_buff as f32 / player.stats.total_damage as f32;
                    }

                    if attack_result.damage >= encounter.boss.current_hp {
                        encounter.boss.current_hp = 0;
                        let damage = attack_result.damage - encounter.boss.current_hp;
                        encounter.stats.total_damage += damage;
                    }
                    else {
                        encounter.boss.current_hp = encounter.boss.current_hp - attack_result.damage;
                        encounter.boss.hp_percentage = encounter.boss.current_hp as f32 / encounter.boss.max_hp as f32;
                        encounter.stats.total_damage += attack_result.damage;
                    }
                }
            }
        }

       

        &self.encounter
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;


}