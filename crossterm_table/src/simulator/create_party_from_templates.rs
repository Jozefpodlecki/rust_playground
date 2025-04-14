use std::collections::HashMap;

use crate::models::PlayerTemplate;

use super::*;



impl Simulator {

    fn apply_cooldown_reduction(cooldown: Duration, value: f32) -> Duration {
        let reduction_factor = 1.0 - value;
        let reduced_cooldown = cooldown.num_nanoseconds().unwrap() as f32 * reduction_factor;
        let reduced_cooldown = Duration::nanoseconds(reduced_cooldown as i64);
        reduced_cooldown
    }
    
    pub fn create_party_from_templates(
        player_templates_map: &mut HashMap<u64, PlayerTemplate>,
        player_states: &mut HashMap<u64, PlayerState>,
        party_states : &mut HashMap<u64, PartyState>,
        player_templates: &mut [PlayerTemplate]
    ) -> Vec<Party> {
        
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
                    skill.cooldown = Self::apply_cooldown_reduction(skill.cooldown, template.cooldown_reduction);
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
                    name: random_alphabetic_string_capitalized(12),
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

        parties
    }
}