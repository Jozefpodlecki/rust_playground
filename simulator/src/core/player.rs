use std::{collections::HashMap, sync::Arc, thread, time::Duration};

use chrono::{DateTime, Utc};
use crossbeam::channel::Sender;
use rand::{rng, Rng};
use strum_macros::{AsRefStr, EnumString};

use crate::core::{event::SimulatorEvent, types::SimulatorContext, utils::create_basic_skills};

#[derive(Debug, Copy, Clone, AsRefStr, PartialEq, EnumString)]
pub enum Class {
    Bard,
    Berserk,
    Sorceress,
    Gunslinger
}

pub struct SimulatorPlayerArgs {
    pub id: u64,
    pub name: String,
}

pub trait SimulatorPlayer {
    fn run(&mut self, context: Arc<SimulatorContext>, sender: Sender<SimulatorEvent>);
}

pub struct SimulatorPlayerBase {
    pub id: u64,
    pub name: String,
    pub class_id: Class,
    pub attack_power: i64,
    pub crit_rate: f64,
    pub crit_damage: f64,
    pub cooldown_reduction: f64,
    pub skills: HashMap<u32, SimulatorPlayerSkill>,
    pub buffs: HashMap<u32, SimulatorPlayerSkillBuff>,
    pub cooldowns: HashMap<u32, DateTime<Utc>>
}

pub struct SimulatorPlayerSkill {
    pub id: u32,
    pub deals_damage: bool,
    pub effects: Vec<SimulatorPlayerSkillEffect>,
    pub cooldown: chrono::Duration,
}

pub struct SimulatorPlayerSkillBuff {
    id: u32,
    expires_on: DateTime<Utc>
}

pub enum SimulatorPlayerSkillBuffType {
    DamageAdditive(f32),
    DamageMultiply(f32)
}

pub enum SimulatorPlayerSkillBuffTarget {
    SelfTarget,
    Party,
}

pub enum SimulatorPlayerSkillBuffCategory {
    Buff,
    Debuff,
}

pub enum SimulatorPlayerSkillEffectType {
    Summon {
        npc_id: u32,
    },
    Buff {
        id: u32,
        buff_type: SimulatorPlayerSkillBuffType,
        target: SimulatorPlayerSkillBuffTarget,
        category: SimulatorPlayerSkillBuffCategory
    }
}

pub struct SimulatorPlayerSkillEffect {
    id: u32,
    effect_type: SimulatorPlayerSkillEffectType
}

pub struct BerserkerSimulatorPlayer {
    base: Option<SimulatorPlayerBase>,
    handle: Option<thread::JoinHandle<()>>
}

impl BerserkerSimulatorPlayer {
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
                class_id: Class::Berserk,
                attack_power: 12000,
                crit_rate: 0.2,
                crit_damage: 2.0,
                cooldown_reduction: 0.1,
                skills,
                cooldowns: HashMap::new(),
                buffs: HashMap::new(),
            }),
        }
    }
}


impl SimulatorPlayer for BerserkerSimulatorPlayer {
    fn run(&mut self, context: Arc<SimulatorContext>, sender: Sender<SimulatorEvent>) {
        let base = self.base.take().unwrap();
        
        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_secs(1));
            let mut rng = rng();
            loop {
                let damage = base.attack_power;
                sender.send(SimulatorEvent::SkillDamage {
                    source_id: base.id as u64,
                    skill_id: 100,
                    damage,
                    target_id: 0, // boss
                }).unwrap();
                thread::sleep(Duration::from_millis(500 + rng.random_range(0..200)));
            }
        });

        self.handle = Some(handle);
    }
}


pub struct SorceressSimulatorPlayer {
    base: Option<SimulatorPlayerBase>,
    handle: Option<thread::JoinHandle<()>>
}

impl SorceressSimulatorPlayer {
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
                class_id: Class::Sorceress,
                attack_power: 12000,
                crit_rate: 0.2,
                crit_damage: 2.0,
                cooldown_reduction: 0.1,
                skills,
                cooldowns: HashMap::new(),
                buffs: HashMap::new(),
            }),
        }
    }
}

pub struct GunslingerSimulatorPlayer {
    base: Option<SimulatorPlayerBase>,
    handle: Option<thread::JoinHandle<()>>
}

impl GunslingerSimulatorPlayer {
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
                class_id: Class::Gunslinger,
                attack_power: 11000,
                crit_rate: 0.25,
                crit_damage: 1.8,
                cooldown_reduction: 0.1,
                skills,
                cooldowns: HashMap::new(),
                buffs: HashMap::new()
            }),
        }
    }
}

impl SimulatorPlayer for SorceressSimulatorPlayer {
    fn run(&mut self, context: Arc<SimulatorContext>, sender: Sender<SimulatorEvent>) {
        let base = self.base.take().unwrap();
        
        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_secs(1));
            let mut rng = rng();
            loop {
                let damage = base.attack_power;
                sender.send(SimulatorEvent::SkillDamage {
                    source_id: base.id as u64,
                    skill_id: 100,
                    damage,
                    target_id: 0, // boss
                }).unwrap();
                thread::sleep(Duration::from_millis(500 + rng.random_range(0..200)));
            }
        });

        self.handle = Some(handle);
    }
}

impl SimulatorPlayer for GunslingerSimulatorPlayer {
    fn run(&mut self, context: Arc<SimulatorContext>, sender: Sender<SimulatorEvent>) {
        let base = self.base.take().unwrap();
        
        let handle = thread::spawn(move || {
            
            let mut rng = rng();
            loop {
                let damage = base.attack_power;

                let skill = base.skills.get(&100).unwrap();

                sender.send(SimulatorEvent::SkillDamage {
                    source_id: base.id as u64,
                    skill_id: 101,
                    damage,
                    target_id: 0,
                }).unwrap();
                thread::sleep(Duration::from_millis(500 + rng.random_range(0..200)));
            }
        });

        self.handle = Some(handle);
    }
}

