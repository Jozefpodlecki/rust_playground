use chrono::Duration;
use std::fmt;
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
    EntityDied {
        id: u64
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

impl fmt::Display for SimulatorEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use SimulatorEvent::*;

        match self {
            NewPlayer { id, name, class_id } => {
                write!(f, "NewPlayer: [id: {}, name: {}, class: {:?}]", id, name, class_id)
            }
            NewParty { id, members } => {
                write!(f, "NewParty: [id: {}, members: {}]", id, members.len())
            }
            NewSummon { id, owner_id } => {
                write!(f, "NewSummon: [id: {}, owner_id: {}]", id, owner_id)
            }
            EntityDied { id } => {
                write!(f, "EntityDied: [id: {}]", id)
            }
            NewBoss { id } => {
                write!(f, "NewBoss: [id: {}]", id)
            }
            Remove { id } => {
                write!(f, "Remove: [id: {}]", id)
            }
            RemoveBuff { id } => {
                write!(f, "RemoveBuff: [id: {}]", id)
            }
            PartyBuff { id, buff_type, source_id, target_id, duration } => {
                write!(f, "PartyBuff: [id: {}, type: {:?}, from: {}, to: {}, duration: {}s]", id, buff_type, source_id, target_id, duration.num_seconds())
            }
            Buff { id, buff_type, source_id, target_id, duration } => {
                write!(f, "Buff: [id: {}, type: {:?}, from: {}, to: {}, duration: {}s]", id, buff_type, source_id, target_id, duration.num_seconds())
            }
            RaidComplete => {
                write!(f, "RaidComplete")
            }
            BossDead { id } => {
                write!(f, "BossDead: [id: {}]", id)
            }
            SkillDamage { source_id, skill_id, current_hp, max_hp, is_critical, damage, target_id } => {
                write!(
                    f,
                    "SkillDamage: [from: {}, to: {}, skill: {}, dmg: {}, crit: {}, target_hp: {}/{}]",
                    source_id, target_id, skill_id, damage, is_critical, current_hp, max_hp
                )
            }
        }
    }
}
