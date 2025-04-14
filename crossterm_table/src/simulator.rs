use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rand::{random_bool, Rng};

use crate::{models::{player_template::SkillType, *}, utils::*};


pub struct Simulator {
    encounter: Encounter,
    started_on: DateTime<Utc>,
    player_templates: HashMap<u64, PlayerTemplate>,
}

impl Simulator {
    pub fn new(
        encounter_template: EncounterTemplate,
        mut player_templates: Vec<PlayerTemplate>) -> Self {

        let mut player_templates_map: HashMap<u64, PlayerTemplate> = HashMap::new();
        let started_on = Utc::now();

        let mut parties = Vec::new();

        for chunk in player_templates.chunks_mut(4) {
            let mut party = Vec::new();

            for template in chunk {

                let mut id = random_number_in_range(1000..9999);

                while player_templates_map.contains_key(&id) {
                    id = random_number_in_range(1000..9999);
                }

                for skill in template.skills.iter_mut() {
                    skill.cooldown = template.cooldown_reduction as f32 * skill.cooldown;
                }

                player_templates_map.insert(id, template.clone());

                party.push(
                    Player {
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
                });
            }
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
        };

        Self {
            encounter,
            started_on,
            player_templates: player_templates_map
        }
    }

    pub fn start(&mut self) {
        self.started_on = Utc::now();
    }

    pub fn perform_attack(
        duration_seconds: i64,
        player_template: &PlayerTemplate) -> Option<AttackResult> {
        let mut rng = rand::rng();

        let min = player_template.attack_power as f32 * 0.8;
        let max = player_template.attack_power as f32 * 1.2;

        let damage = rng.random_range(min..max);
        let mut is_critical = false;

        let length = player_template.skills.len();

        let random_index = rng.random_range(0..length);
        let skill_template = player_template.skills[random_index];

        if skill_template.kind == SkillType::Awakening {
            return AttackResult {
                skill_id: skill_template.id,
                damage: 0,
                is_critical: false,
            };
        }

        let skill_id = skill_template.id;

        let damage = rng.random_bool(player_template.crit_rate as f64).then(|| {
            is_critical = true;
            damage as u64 * 2
        }).unwrap_or_else(|| damage as u64);

        AttackResult {
            skill_id,
            damage: damage as u64,
            is_critical,
        }
    }

    pub fn tick(&mut self) -> &Encounter {

        let now = Utc::now();

        let elapsed_duration = now - self.started_on;
        let duration_seconds = elapsed_duration.num_seconds();
        let formatted_duration = format_duration(duration_seconds);

        self.encounter.duration = EncounterDuration {
            elapsed_seconds: duration_seconds as u64,
            mmss: formatted_duration,
        };

        for party in &mut self.encounter.parties {
            for player in party {
                
                let player_template = self.player_templates.get(&player.id).unwrap();
                let damage = Self::perform_attack(
                    duration_seconds,
                    player_template);
                player.stats.total_damage = damage;
                total_damage += damage;
            }
        }

        self.encounter.boss.current_hp = self.encounter.boss.current_hp.saturating_sub(total_damage);

        &self.encounter
    }
}