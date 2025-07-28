use std::sync::{Arc, Barrier, RwLock};

use crate::core::player::{Class, SimulatorPlayer};

pub struct EncounterTemplate {
    pub boss: EncounterTemplateBoss,
    pub parties: Vec<EncounterTemplateParty>
}

pub struct EncounterTemplateBoss {
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
}

pub struct SimulatorContext {
    pub barrier: Arc<Barrier>,
    pub current_boss: RwLock<SimulatorBoss>
}

pub struct SimulatorBoss {
    pub id: u64,
    pub hp_bars: u32,
    pub current_hp: i64,
    pub max_hp: i64
}

pub struct SimulatorParty {
    pub id: u32,
    pub members: Vec<Box<dyn SimulatorPlayer>>
}