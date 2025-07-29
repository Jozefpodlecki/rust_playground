use std::{collections::HashMap, sync::{Arc, Barrier, RwLock}};

use chrono::{DateTime, Utc};

use crate::core::{boss::SimulatorBoss, player::{Class, SimulatorPlayer}};

pub struct EncounterTemplate {
    pub boss: EncounterTemplateBoss,
    pub parties: Vec<EncounterTemplateParty>
}

pub struct EncounterTemplateBoss {
    pub id: u64,
    pub npc_id: u32,
    pub hp_bars: u16,
    pub max_hp: i64,
    pub summons: Vec<EncounterTemplateBossSummon>
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub enum EncounterTemplateBossSummonConditon {
    HpBars(u16),
    Death
}

pub struct EncounterTemplateBossSummon {
    pub id: u64,
    pub condition: EncounterTemplateBossSummonConditon,
    pub npc_id: u32,
    pub hp_bars: u16,
    pub max_hp: i64,
}

pub struct EncounterTemplateParty {
    pub id: u32,
    pub members: Vec<EncounterTemplatePartyMember>
}

#[derive(Debug, Clone)]
pub struct EncounterTemplatePartyMember {
    pub id: u64,
    pub name: String,
    pub class_id: Class,
    pub attack_power: i64,
    pub crit_rate: f64,
    pub crit_damage: f64,
    pub cooldown_reduction: f64,
}

pub struct SimulatorContext {
    pub barrier: Arc<Barrier>,
    pub party_map: HashMap<u32, Vec<u64>>,
    pub player_ids: Vec<u64>,
    pub current_boss: RwLock<SimulatorBoss>
}

pub struct SimulatorParty {
    pub id: u32,
    pub members: Vec<Box<dyn SimulatorPlayer>>
}