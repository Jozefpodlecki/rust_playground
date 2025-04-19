mod create_party_from_templates;
mod stats;
mod id_generator;
mod skills_manager;

use std::{
    sync::{atomic::{AtomicBool, Ordering}, mpsc::{self, Receiver}, Arc, Mutex, RwLock},
    thread::{self, sleep}, time::Duration,
};
use chrono::{DateTime, Utc};
use id_generator::IdGenerator;
use uuid::Uuid;
use std::collections::HashMap;

use crate::{models::{player_template::*, *}, utils::random_number_in_range};

#[derive(Default)]
pub struct Simulator {
    encounter: Encounter,
    player_templates: HashMap<u64, PlayerTemplate>,
    control_flag: Arc<AtomicBool>
}

impl Simulator {
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
            is_cleared: false,
            boss: Boss { 
                id: id_generator.next_npc_id(),
                name: encounter_template.boss.name,
                max_hp: encounter_template.boss.max_hp,
                current_hp: encounter_template.boss.max_hp,
                hp_percentage: 1.0,
                max_hp_bars: encounter_template.boss.hp_bars,
                hp_bars: encounter_template.boss.hp_bars as f32,
                bar_per_hp: encounter_template.boss.max_hp as f32 / encounter_template.boss.hp_bars as f32
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

        let control_flag = Arc::new(AtomicBool::new(false));
        
        Self {
            encounter,
            player_templates: player_templates_map,
            control_flag,
            ..Default::default()
        }
    }

    pub fn start(&mut self) {
        let started_on = Utc::now();
        self.encounter.started_on = started_on;
        self.encounter.id = Uuid::now_v7();
        
        let (tx, rx) = crossbeam::channel::unbounded::<Message>();
        let control_flag = self.control_flag.clone();
        self.control_flag.store(true, Ordering::Release);
    }

    pub fn has_ended(&mut self) -> bool {
        true
    }

    // pub fn get_encounter(&mut self, timeout: Duration) -> &Encounter {
    //     match self.rx.as_ref().unwrap().recv_timeout(timeout) {
    //         Ok(encounter) => {
    //             self.encounter = encounter;
    //             &self.encounter
    //         },
    //         Err(_) => &self.encounter,
    //     }
    // }

   

}