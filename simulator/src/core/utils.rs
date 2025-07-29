use std::{collections::HashMap};

use chrono::Duration;

use crate::core::player::*;

pub fn create_bard_skills(base_id: u32, attack_power: i64) -> HashMap<u32, SimulatorPlayerSkill> {

    let mut map = HashMap::new();

    map.insert(100, SimulatorPlayerSkill { 
            id: 100,
            deals_damage: false,
            identity_gain: 0.1,
            min_damage: 1.0,
            max_damage: 2.0,
            effects: vec![
                SimulatorPlayerSkillEffect::Buff { 
                    id: 1000,
                    buff_id: 100,
                    buff_type: SimulatorPlayerSkillBuffType::DamageAdditive(0.15 * attack_power as f32),
                    target: SimulatorPlayerSkillBuffTarget::Party,
                    category: SimulatorPlayerSkillBuffCategory::Buff,
                    duration: Duration::seconds(7)
                }
            ],
            cooldown: Duration::seconds(10)
        }
    );

    map.insert(101, SimulatorPlayerSkill { 
            id: 101,
            identity_gain: 0.4,
            deals_damage: false,
            min_damage: 1.0,
            max_damage: 2.0,
            effects: vec![],
            cooldown: Duration::seconds(18)
        }
    );

    map.insert(102, SimulatorPlayerSkill { 
            id: 102,
            deals_damage: false,
            identity_gain: 0.4,
            min_damage: 0.1,
            max_damage: 0.2,
            effects: vec![
                SimulatorPlayerSkillEffect::Summon {
                    id: 1001,
                    npc_id: 1001,
                    duration: Duration::seconds(10),
                    effects: vec![
                        SimulatorPlayerSkillEffect::Buff {
                            id: 1002,
                            buff_id: 1,
                            buff_type: SimulatorPlayerSkillBuffType::DamageMultiply(0.1),
                            target: SimulatorPlayerSkillBuffTarget::SelfTarget,
                            category: SimulatorPlayerSkillBuffCategory::Debuff,
                            duration: Duration::seconds(5)
                        }
                    ]
                }
            ],
            cooldown: Duration::seconds(24)
        }
    );

    map
}

pub fn create_basic_skills(base_id: u32) -> HashMap<u32, SimulatorPlayerSkill> {
    let cooldowns = [5, 10, 15, 20];
    cooldowns.iter().enumerate().map(|(i, secs)| {
        let id = base_id + i as u32;
        (
            id,
            SimulatorPlayerSkill {
                id,
                identity_gain: 0.1,
                min_damage: 10.0,
                max_damage: 20.0,
                deals_damage: true,
                effects: vec![],
                cooldown: Duration::seconds(*secs),
            },
        )
    }).collect()
}