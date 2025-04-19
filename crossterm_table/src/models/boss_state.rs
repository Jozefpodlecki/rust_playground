use std::sync::{Arc, RwLock};

use chrono::{DateTime, Utc};
use crossbeam::channel::Sender;

use super::Buff;


#[derive(Default)]
pub struct BossState {
    pub id: u64,
    pub current_hp: u64,
    pub active_debuffs: Vec<Buff>,
}

impl BossState {
    pub fn add_debuff(&mut self, new_buff: Buff) {
        self.active_debuffs.retain(|buff| buff.kind != new_buff.kind);
        self.active_debuffs.push(new_buff);
    }

    pub fn refresh(&mut self, now: DateTime<Utc>) {
        self.active_debuffs.retain(|pr| pr.expires_on > now);
    }
}

pub enum BossCommand {

}

#[derive(Clone)]
pub struct BossHandle {
    pub_state: Arc<RwLock<BossState>>,
    command_tx: Sender<BossCommand>,
}