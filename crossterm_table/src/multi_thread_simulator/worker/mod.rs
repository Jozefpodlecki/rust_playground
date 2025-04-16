use std::{
    collections::HashMap, sync::{atomic::{AtomicBool, Ordering}, mpsc::{self, Receiver, Sender}, Arc, Mutex, RwLock}, thread::{self, sleep, JoinHandle}
};

use artist::ArtistWorker;
use bard::BardWorker;
use chrono::{DateTime, Utc};
use generic::{GenericWorker};
use paladin::PaladinWorker;
use slayer::SlayerWorker;

use crate::models::{class::Class, player_template::*, *};
use crate::multi_thread_simulator::worker::on_attack_result::on_attack_result;

use super::stats::*;

mod generic;
mod bard;
mod slayer;
mod paladin;
mod artist;
mod on_attack_result;

pub trait Worker {
    fn start_loop(&mut self);
}

pub fn get_worker_from_class(
    template: PlayerTemplate,
    party_state: Arc<RwLock<PartyState>>,
    boss_state: Arc<RwLock<BossState>>,
    started_on: DateTime<Utc>,
    player_id: u64,
    tx: Sender<AttackResult>,
    start_flag: Arc<AtomicBool>,
) -> Box<dyn Worker> {
    match template.class {
        Class::Bard => Box::new(BardWorker::new(template, party_state, boss_state, started_on, player_id, tx, start_flag)),
        Class::Slayer => Box::new(SlayerWorker::new(template, party_state, boss_state, started_on, player_id, tx, start_flag)),
        Class::Artist => Box::new(ArtistWorker::new(template, party_state, boss_state, started_on, player_id, tx, start_flag)),
        Class::Paladin => Box::new(PaladinWorker::new(template, party_state, boss_state, started_on, player_id, tx, start_flag)),
        _ => Box::new(GenericWorker::new(template, party_state, boss_state, started_on, player_id, tx, start_flag)),
    }
}

pub fn spawn_player_threads(
    player_templates: &HashMap<u64, PlayerTemplate>,
    encounter: &Encounter,
    boss_state: Arc<RwLock<BossState>>,
    tx: Sender<AttackResult>,
    start_flag: Arc<AtomicBool>,
) -> Vec<JoinHandle<()>> {
    let started_on = encounter.started_on;
    let mut worker_threads: Vec<JoinHandle<()>> = Vec::new();

    for party in &encounter.parties {
        let party_state = Arc::new(RwLock::new(PartyState::default()));

        for player in &party.players {
            let template = player_templates.get(&player.id).unwrap().clone();
            let tx = tx.clone();
            let start_flag = start_flag.clone();
            let boss_state = boss_state.clone();
            let party_state = party_state.clone();
            let player_id = player.id;

            let handle = thread::spawn(move || {
                let mut worker = get_worker_from_class(
                    template,
                    party_state,
                    boss_state,
                    started_on,
                    player_id,
                    tx.clone(),
                    start_flag
                );

                worker.start_loop();
            });

            worker_threads.push(handle);
        }
    }

    worker_threads
}

pub fn spawn_result_listener_thread(
    rx: mpsc::Receiver<AttackResult>,
    encounter: Encounter,
    boss_state: Arc<RwLock<BossState>>) -> Receiver<Encounter> {
    
    let (tx, rx_encounter) = mpsc::channel::<Encounter>();
    let mut encounter = encounter.clone();
    let started_on = encounter.started_on;

    thread::spawn(move || {
        while let Ok(attack_result) = rx.recv() {
            on_attack_result(
                started_on,
                &mut encounter,
                &boss_state,
                attack_result);
            
            tx.send(encounter.clone());
        }
    });

    rx_encounter
}