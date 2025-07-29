use chrono::Duration;

use crate::core::player::{Class, SimulatorPlayerBase, SimulatorPlayerSkillBuffType};

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
    NewSummon {
        id: u64,
        owner_id: u64
    },
    NewBoss {
        id: u64
    },
    Remove {
        id: u64
    },
    RemoveBuff {
        id: u32
    },
    PartyBuff {
        id: u32,
        buff_type: SimulatorPlayerSkillBuffType,
        source_id: u64,
        target_id: u64,
        duration: Duration
    },
    Buff {
        id: u32,
        buff_type: SimulatorPlayerSkillBuffType,
        source_id: u64,
        target_id: u64,
        duration: Duration
    },
    RaidComplete {

    },
    BossDead {
        id: u64
    },
    SkillDamage {
        source_id: u64, 
        skill_id: u32,
        current_hp: i64,
        max_hp: i64,
        is_critical: bool,
        damage: i64,
        target_id: u64
    }
}
