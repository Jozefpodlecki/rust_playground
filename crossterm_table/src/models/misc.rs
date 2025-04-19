use std::collections::HashMap;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::Buff;


pub struct BossTemplate {
    pub name: &'static str,
    pub max_hp: u64,
    pub hp_bars: u64
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum HitOption {
    #[default]
    None,
    Back,
    Frontal,
    Flank,
}

pub enum Message {
    Attack
}

#[derive(Default, Debug, Clone)]
pub struct AttackResult {
    pub source_id: u64,
    pub target_id: u64,
    pub hit_option: HitOption,
    pub skill_id: u32,
    pub damage: u64,
    pub is_critical: bool,
    pub is_hyper_awakening: bool,
    pub with_brand: bool,
    pub with_attack_power_buff: bool,
    pub with_identity_buff: bool,
    pub with_hat_buff: bool,
}


#[derive(Default)]
pub struct PlayerState {
    pub skill_cooldowns: HashMap<u32, DateTime<Utc>>,
    pub active_buffs: HashMap<u32, Buff>,
    pub identity: f32
}


#[derive(Default)]
pub struct PartyState {
    pub active_buffs: HashMap<u32, Buff>,
}
