mod create_party_from_templates;
mod apply_buffs;
mod stats;
mod attack;
mod worker;
mod id_generator;

use std::{
    sync::{atomic::{AtomicBool, Ordering}, mpsc, Arc, Mutex, RwLock},
    thread::{self, sleep},
};
use chrono::{DateTime, Utc};
use id_generator::IdGenerator;
use uuid::Uuid;
use worker::*;
use std::collections::HashMap;

use crate::{models::{player_template::*, *}, utils::random_number_in_range};

#[derive(Default)]
pub struct MultiThreadSimulator {
    encounter: Arc<Mutex<Encounter>>,
    player_templates: HashMap<u64, PlayerTemplate>,
    start_flag: Arc<AtomicBool>,
}

impl MultiThreadSimulator {
    pub fn new(
        encounter_template: EncounterTemplate,
        mut player_templates: Vec<PlayerTemplate>,
    ) -> Self {
        let mut id_generator = IdGenerator::new();
        let mut player_templates_map: HashMap<u64, PlayerTemplate> = HashMap::new();
        
        let parties = Self::create_party_from_templates(
            &mut id_generator,
            &mut player_templates_map,
            &mut player_templates);

        let encounter = Encounter {
            id: Uuid::nil(),
            boss: Boss { 
                id: id_generator.next_npc_id(),
                name: encounter_template.boss.name,
                max_hp: encounter_template.boss.max_hp,
                current_hp: encounter_template.boss.max_hp,
                hp_percentage: 1.0,
                hp_bars: encounter_template.boss.hp_bars
            },
            duration: EncounterDuration {
                elapsed_seconds: 0,
                mmss: "00:00".to_string(),
            },
            started_on: DateTime::<Utc>::MIN_UTC,
            parties,
            stats: EncounterStats { 
                total_damage: 0,
                ttk: "INF".to_string(),
                dps: 0
            }
        };

        let start_flag = Arc::new(AtomicBool::new(false));
        let encounter= Arc::new(Mutex::new(encounter));
        
        Self {
            encounter,
            player_templates: player_templates_map,
            start_flag,
        }
    }

    pub fn start(&mut self) {
        let started_on = Utc::now();
        {
            let mut encounter = self.encounter.lock().unwrap();
            encounter.started_on = started_on;
            encounter.id = Uuid::now_v7();
        }
        

        let (tx, rx) = mpsc::channel::<AttackResult>();
        let start_flag = self.start_flag.clone();
        let boss_state: Arc<RwLock<BossState>> = {
            let encounter = self.encounter.lock().unwrap();
            let state = BossState {
                id: encounter.boss.id,
                current_hp: encounter.boss.current_hp,
                active_debuffs: HashMap::new()
            };
            Arc::new(RwLock::new(state))
        };

        spawn_player_threads(
            &self.player_templates,
            self.encounter.clone(),
            boss_state.clone(),
            tx,
            start_flag.clone());
        spawn_result_listener_thread(
            rx,
            self.encounter.clone(),
            boss_state.clone());

        self.start_flag.store(true, Ordering::Release);
    }

    pub fn get_encounter(&mut self) -> Encounter {
        self.encounter.lock().unwrap().clone()
    }

   

}


// {
        //     let cloned_encounter = self.encounter.clone();
        //     let encounter = cloned_encounter.lock().unwrap();
        //     let started_on = encounter.started_on;

        //     for party in encounter.parties.clone() {

        //         let party_state = Arc::new(RwLock::new(PartyState { active_buffs: HashMap::new() }));
    
        //         for player in party.players.clone() {
        //             let player_template = self.player_templates.clone();
                  
        //             let tx = tx.clone();
        //             let start_flag = start_flag.clone();
        //             let boss_state = boss_state.clone();
        //             let party_state: Arc<RwLock<PartyState>> = party_state.clone();
    
        //             thread::spawn(move || {
        //                 while !start_flag.load(Ordering::Acquire) {
        //                     thread::yield_now();
        //                 }
    
        //                 let player_id = player.id;
        //                 let mut player_state = PlayerState::default();
        //                 let player_template = player_template.get(&player.id).unwrap();
        //                 let current_hp = boss_state.read().unwrap().current_hp;
    
        //                 while current_hp > 0 {
        //                     let now = Utc::now();
        //                     let elapsed_duration = now - started_on;
        //                     let duration_seconds = elapsed_duration.num_seconds();

        //                     let attack_result = perform_attack(
        //                         player_id,
        //                         now,
        //                         boss_state.clone(),
        //                         party_state.clone(),
        //                         &mut player_state,
        //                         duration_seconds,
        //                         player_template
        //                     );
    
        //                     if let Some(result) = attack_result {
        //                         tx.send(result).unwrap();
        //                     }
    
        //                     let next_cooldown = player_state.skill_cooldowns.values()
        //                         .min()
        //                         .unwrap_or(&now);
    
        //                     let sleep_duration = (*next_cooldown - now).max(chrono::Duration::seconds(0));
        //                     thread::sleep(sleep_duration.to_std().unwrap());
        //                 }
        //             });
        //         }
        //     }
        // }