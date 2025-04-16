use std::{collections::HashMap, sync::{atomic::{AtomicBool, Ordering}, mpsc::Sender, Arc, RwLock}, thread::{self, sleep}};

use chrono::{DateTime, Utc};

use crate::{models::{AttackResult, BossState, Buff, PartyState, PlayerTemplate}, multi_thread_simulator::attack::perform_attack};

pub trait Worker {
    fn start_loop(&mut self);
}

pub struct GenericWorker {
    skill_cooldowns: HashMap<u32, DateTime<Utc>>,
    active_buffs: HashMap<u32, Buff>,
    identity: f32,
    party_state: Arc<RwLock<PartyState>>,
    boss_state: Arc<RwLock<BossState>>,
    template: PlayerTemplate,
    started_on: DateTime<Utc>,
    player_id: u64,
    tx: Sender<AttackResult>,
    control_flag: Arc<AtomicBool>,
}

impl GenericWorker {
    pub fn new(
        template: PlayerTemplate,
        party_state: Arc<RwLock<PartyState>>,
        boss_state: Arc<RwLock<BossState>>,
        started_on: DateTime<Utc>,
        player_id: u64,
        tx: Sender<AttackResult>,
        control_flag: Arc<AtomicBool>,
    ) -> Self {
        Self {
            skill_cooldowns: HashMap::new(),
            active_buffs: HashMap::new(),
            identity: 0.0,
            party_state,
            boss_state,
            template,
            started_on,
            player_id,
            tx,
            control_flag,
        }
    }
}

impl Worker for GenericWorker {
    fn start_loop(&mut self) {

        while !self.control_flag.load(Ordering::Acquire) {
            thread::yield_now();
        }

        while self.boss_state.read().unwrap().current_hp > 0 {
            if !self.control_flag.load(Ordering::Relaxed) {
                break;
            }

            let now = Utc::now();
            let duration = (now - self.started_on).num_seconds();

            // if let Some(result) = perform_attack(
            //     self.player_id,
            //     now,
            //     self.boss_state.clone(),
            //     self.party_state.clone(),
            //     &mut self,
            //     duration,
            //     &self.template,
            // ) {
            //     self.tx.send(result).unwrap();
            // }

            if let Some(next_cooldown) = self.skill_cooldowns.values().min() {
                let duration = (*next_cooldown - now).to_std().unwrap();
                sleep(duration);
            }
        }
    }
}



