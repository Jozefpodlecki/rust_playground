
use std::{collections::HashMap, sync::Arc, thread, time::Duration};

use crossbeam::channel::Sender;

use crate::core::{event::SimulatorEvent, player::*, types::SimulatorContext, utils::create_basic_skills};

pub enum BardSkills {
    Buff = 100
}

pub struct BardSimulatorPlayer {
    base: Option<SimulatorPlayerBase>,
    handle: Option<thread::JoinHandle<()>>
}

impl BardSimulatorPlayer {
    pub fn new(args: SimulatorPlayerArgs) -> Self {

        let SimulatorPlayerArgs {
            id,
            name
        }= args;

        let skills = create_basic_skills(100);

        Self {
            handle: None,
            base: Some(SimulatorPlayerBase {
                id: id,
                name,
                class_id: Class::Bard,
                attack_power: 0,
                crit_rate: 0.0,
                crit_damage: 0.0,
                cooldown_reduction: 0.0,
                skills,
                cooldowns: HashMap::new(),
                buffs: HashMap::new(),
            }),
        }
    }
}

impl SimulatorPlayer for BardSimulatorPlayer {
    fn run(&mut self, context: Arc<SimulatorContext>, sender: Sender<SimulatorEvent>) {
        let base = self.base.take().unwrap();

        let handle = thread::spawn(move || {

            loop {

                if context.current_boss.read().unwrap().current_hp <= 0 {
                    
                }

                thread::sleep(Duration::from_secs(2));
            }
        });

        self.handle = Some(handle);
    }
}