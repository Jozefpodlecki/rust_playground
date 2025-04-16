use std::collections::HashMap;

use crate::models::PlayerTemplate;

use super::*;

impl Simulator {
    pub fn apply_buffs(
        attack_power: u64,
        buffs: &Vec<BuffTemplate>,
        now: DateTime<Utc>,
        party_state: &mut PartyState,
        boss_state: &mut BossState,
        player_state: &mut PlayerState) {

        for buff_template in buffs {
            let expires_on = now + buff_template.duration;
            let buff = Buff {
                target: buff_template.target,
                kind: buff_template.kind,
                expires_on,
                value: attack_power as f32
            };

            if buff_template.category == BuffCategory::Debuff {
                let mut instance_id = random_number_in_range(1000..9999);

                while boss_state.active_debuffs.contains_key(&instance_id) {
                    instance_id = random_number_in_range(1000..9999);
                }

                boss_state.active_debuffs.insert(instance_id, buff.clone());
            }

            if buff_template.category == BuffCategory::Buff {

                if buff_template.target == BuffTarget::TargetSelf {
                    let mut instance_id = random_number_in_range(1000..9999);

                    while player_state.active_buffs.contains_key(&instance_id) {
                        instance_id = random_number_in_range(1000..9999);
                    }
        
                    player_state.active_buffs.insert(
                        instance_id,
                        buff.clone()
                    );   
                }

                if buff_template.target == BuffTarget::Party {
                    let mut instance_id = random_number_in_range(1000..9999);

                    while party_state.active_buffs.contains_key(&instance_id) {
                        instance_id = random_number_in_range(1000..9999);
                    }
        
                    party_state.active_buffs.insert(
                        instance_id,
                        buff
                    );   
                }

                
            }
        }
    }

}