use crate::core::player::{Class, SimulatorPlayerBase};

#[derive(Debug)]
pub enum SimulatorEvent {
    NewPlayer {
        id: u64,
        name: String,
        class_id: Class
    },
    NewParty {
        id: u32,
        members: Vec<SimulatorPlayerBase>
    },
    NewBoss {
        id: u64
    },
    RaidComplete {

    },
    BossDead {
        id: u64
    },
    SkillDamage {
        source_id: u64, 
        skill_id: u32,
        damage: i64,
        target_id: u64
    }
}
