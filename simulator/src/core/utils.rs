use std::collections::HashMap;

use chrono::Duration;

use crate::core::player::SimulatorPlayerSkill;

pub fn create_basic_skills(base_id: u32) -> HashMap<u32, SimulatorPlayerSkill> {
    let cooldowns = [5, 10, 15, 20];
    cooldowns.iter().enumerate().map(|(i, secs)| {
        let id = base_id + i as u32;
        (
            id,
            SimulatorPlayerSkill {
                id,
                deals_damage: true,
                effects: vec![],
                cooldown: Duration::seconds(*secs),
            },
        )
    }).collect()
}