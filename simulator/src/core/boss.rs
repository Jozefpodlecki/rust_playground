use std::{sync::Arc, thread, time::Duration};

use crossbeam::channel::{Receiver, Sender};
use rand::{rng, seq::IndexedRandom};

use crate::core::{event::SimulatorEvent, types::SimulatorContext};

pub struct SimulatorBoss {
    pub id: u64,
    pub hp_bars: u32,
    pub current_hp: i64,
    pub max_hp: i64,
    handle: Option<thread::JoinHandle<()>>
}

impl SimulatorBoss {
    pub fn new() -> Self {
        Self {
            current_hp: 0,
            hp_bars: 0,
            id: 0,
            max_hp: 0,
            handle: None,
        }
    }

    pub fn run(&mut self, context: Arc<SimulatorContext>, receiver: Receiver<SimulatorEvent>, sender: Sender<SimulatorEvent>) {
        let id = self.id;
        
        let handle = thread::spawn(move || {
            
            let mut rng = rng();

            context.barrier.wait();

            let duration = Duration::from_secs(1);
            

            loop {
                if let Ok(event) = receiver.recv_timeout(duration) {

                }

                let target_id = *context.player_ids.choose(&mut rng).unwrap();

                let event = SimulatorEvent::SkillDamage { 
                    damage: 0,
                    skill_id: 0,
                    source_id: id,
                    target_id,
                };

                sender.send(event).unwrap();
            }
        });

        self.handle = Some(handle);
    }
}