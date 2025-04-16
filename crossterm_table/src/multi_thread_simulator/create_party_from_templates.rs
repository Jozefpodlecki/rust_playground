use std::collections::HashMap;

use chrono::Duration;

use crate::{models::PlayerTemplate, utils::{random_alphabetic_string_capitalized, random_number_in_range}};

use super::*;

impl MultiThreadSimulator {

    fn apply_cooldown_reduction(cooldown: Duration, value: f32) -> Duration {
        let reduction_factor = 1.0 - value;
        let reduced_cooldown = cooldown.num_nanoseconds().unwrap() as f32 * reduction_factor;
        let reduced_cooldown = Duration::nanoseconds(reduced_cooldown as i64);
        reduced_cooldown
    }
    
    pub fn create_party_from_templates(
        id_generator: &mut IdGenerator,
        player_templates_map: &mut HashMap<u64, PlayerTemplate>,
        player_templates: &mut [PlayerTemplate]
    ) -> Vec<Party> {
        
        let mut parties = Vec::new();

        for chunk in player_templates.chunks_mut(4) {
            let members = Vec::new();
            let mut party= Party {
                id: id_generator.next_party_id(),
                players: members,
                ..Default::default()
            };

            for template in chunk {
                let id = id_generator.next_player_id();

                for skill in template.skills.iter_mut() {
                    let cooldown_reduction = template.cooldown_reduction + skill.cooldown_gem;
                    skill.cooldown = Self::apply_cooldown_reduction(skill.cooldown, cooldown_reduction);
                }

                player_templates_map.insert(id, template.clone());

                let player = Player {
                    id,
                    name: template.name.clone().unwrap_or_else(|| id_generator.next_player_name(12)),
                    class: template.class.clone(),
                    stats: PlayerStats {
                        skills: PlayerSkillsStats {
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    skills: HashMap::new(),
                };

                party.players.push(player);
            }

            parties.push(party);
        }

        parties
    }
}