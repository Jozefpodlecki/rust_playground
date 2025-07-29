use std::{collections::HashMap, sync::Arc, thread};

use chrono::{DateTime, Duration, Utc};
use crossbeam::channel::{Receiver, Sender};
use log::info;
use rand::{rng, Rng};
use strum_macros::{AsRefStr, EnumString};

use crate::core::{event::SimulatorEvent, types::SimulatorContext, utils::create_basic_skills};

#[derive(Debug, Default, Copy, Clone, AsRefStr, PartialEq, EnumString)]
pub enum Class {
    #[default]
    Unknown,
    Bard,
    Berserk,
    Sorceress,
    Gunslinger
}

pub struct SimulatorPlayerArgs {
    pub id: u64,
    pub name: String,
    pub attack_power: i64,
    pub crit_rate: f64,
    pub crit_damage: f64,
    pub cooldown_reduction: f64,
    pub party_id: u32
}

pub trait SimulatorPlayer {
    fn base(&self) -> SimulatorPlayerBase;
    fn run(&mut self, context: Arc<SimulatorContext>, received: Receiver<SimulatorEvent>, sender: Sender<SimulatorEvent>);
}

#[derive(Debug, Default, Clone)]
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
    pub cooldowns: HashMap<u32, DateTime<Utc>>,
    pub summons: HashMap<u32, SimulatorPlayerSummon>,
    pub party_id: u32
}

#[derive(Debug, Clone)]
pub struct SimulatorPlayerSummon {
    pub id: u64,
    pub npc_id: u32,
    pub next_attack_on: DateTime<Utc>,
    pub expires_on: DateTime<Utc>,
    pub effects: Vec<SimulatorPlayerSkillEffect>
}

#[derive(Debug, Clone)]
pub struct SimulatorPlayerSkill {
    pub id: u32,
    pub identity_gain: f64,
    pub min_damage: f64,
    pub max_damage: f64,
    pub deals_damage: bool,
    pub effects: Vec<SimulatorPlayerSkillEffect>,
    pub cooldown: chrono::Duration,
}


#[derive(Debug, Copy, Clone)]
pub struct SimulatorPlayerSkillBuff {
    pub id: u32,
    pub expires_on: DateTime<Utc>
}


#[derive(Debug, Copy, Clone)]
pub enum SimulatorPlayerSkillBuffType {
    DamageAdditive(f32),
    DamageMultiply(f32)
}


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SimulatorPlayerSkillBuffTarget {
    SelfTarget,
    Party,
}


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SimulatorPlayerSkillBuffCategory {
    Buff,
    Debuff,
}

#[derive(Debug, Clone)]
pub enum SimulatorPlayerSkillEffect {
    Summon {
        id: u32,
        npc_id: u32,
        duration: Duration,
        effects: Vec<SimulatorPlayerSkillEffect>
    },
    Buff {
        id: u32,
        buff_id: u32,
        buff_type: SimulatorPlayerSkillBuffType,
        target: SimulatorPlayerSkillBuffTarget,
        category: SimulatorPlayerSkillBuffCategory,
        duration: Duration
    }
}

pub struct BerserkerSimulatorPlayer {
    base: Option<SimulatorPlayerBase>,
    handle: Option<thread::JoinHandle<()>>
}

impl BerserkerSimulatorPlayer {
    pub fn new(args: SimulatorPlayerArgs) -> Self {

        let SimulatorPlayerArgs {
            id,
            name,
            attack_power,
            cooldown_reduction,
            crit_damage,
            crit_rate,
            party_id
        }= args;

        let skills = create_basic_skills(100);

        Self {
            handle: None,
            base: Some(SimulatorPlayerBase {
                id: id,
                name,
                class_id: Class::Berserk,
                attack_power,
                crit_rate,
                crit_damage,
                cooldown_reduction,
                skills,
                party_id,
                ..Default::default()
            }),
        }
    }
}


impl SimulatorPlayer for BerserkerSimulatorPlayer {

    fn base(&self) -> SimulatorPlayerBase {
        self.base.clone().unwrap()
    }

    fn run(&mut self, context: Arc<SimulatorContext>, receiver: Receiver<SimulatorEvent>, sender: Sender<SimulatorEvent>) {
        let base = self.base.take().unwrap();
        
        let handle = thread::spawn(move || {
            
            let mut rng = rng();
            info!("{}", base.class_id.as_ref());
            context.barrier.wait();

            loop {
                let damage = base.attack_power;

                let skill = base.skills.get(&100).unwrap();
                let current_boss = context.current_boss.read().unwrap();
                let target_id = current_boss.id;

                let damage = base.attack_power;
                sender.send(SimulatorEvent::SkillDamage {
                    source_id: base.id as u64,
                    skill_id: 100,
                    is_critical: false,
                    damage,
                    max_hp: current_boss.max_hp,
                    current_hp: current_boss.current_hp.saturating_sub(damage),
                    target_id: target_id
                }).unwrap();
                thread::sleep(std::time::Duration::from_millis(500 + rng.random_range(0..200)));
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
            name,
            attack_power,
            cooldown_reduction,
            crit_damage,
            crit_rate,
            party_id,
        }= args;

        let skills = create_basic_skills(100);

        Self {
            handle: None,
            base: Some(SimulatorPlayerBase {
                id: id,
                name,
                class_id: Class::Sorceress,
                attack_power,
                crit_rate,
                crit_damage,
                cooldown_reduction,
                skills,
                party_id,
                ..Default::default()
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
            name,
            attack_power,
            cooldown_reduction,
            crit_damage,
            crit_rate,
            party_id
        }= args;

        let skills = create_basic_skills(100);

        Self {
            handle: None,
            base: Some(SimulatorPlayerBase {
                id: id,
                name,
                class_id: Class::Gunslinger,
                attack_power,
                crit_rate,
                crit_damage,
                cooldown_reduction,
                skills,
                party_id,
                ..Default::default()
            }),
        }
    }
}

impl SimulatorPlayer for SorceressSimulatorPlayer {

    fn base(&self) -> SimulatorPlayerBase {
        self.base.clone().unwrap()
    }

    fn run(&mut self, context: Arc<SimulatorContext>, received: Receiver<SimulatorEvent>, sender: Sender<SimulatorEvent>) {
        let base = self.base.take().unwrap();
        
        let handle = thread::spawn(move || {
            
            let mut rng = rng();
            info!("{}", base.class_id.as_ref());
            context.barrier.wait();

            loop {
                let damage = base.attack_power;

                let skill = base.skills.get(&100).unwrap();
                let current_boss = context.current_boss.read().unwrap();
                let target_id = current_boss.id;

                let event  =SimulatorEvent::SkillDamage {
                    source_id: base.id as u64,
                    skill_id: skill.id,
                    max_hp: current_boss.max_hp,
                    current_hp: current_boss.current_hp.saturating_sub(damage),
                    is_critical: false,
                    damage,
                    target_id,
                };

                sender.send(event).unwrap();

                thread::sleep(std::time::Duration::from_millis(500 + rng.random_range(0..200)));
            }
        });

        self.handle = Some(handle);
    }
}

impl SimulatorPlayer for GunslingerSimulatorPlayer {

    fn base(&self) -> SimulatorPlayerBase {
        self.base.clone().unwrap()
    }

    fn run(&mut self, context: Arc<SimulatorContext>, receiver: Receiver<SimulatorEvent>, sender: Sender<SimulatorEvent>) {
        let base = self.base.take().unwrap();
        
        let handle = thread::spawn(move || {
            
            let mut rng = rng();
            info!("{}", base.class_id.as_ref());
            context.barrier.wait();

            loop {
                let damage = base.attack_power;

                let skill = base.skills.get(&100).unwrap();
                let current_boss = context.current_boss.read().unwrap();
                let target_id = current_boss.id;

                let event = SimulatorEvent::SkillDamage {
                    source_id: base.id as u64,
                    skill_id: skill.id,
                    max_hp: current_boss.max_hp,
                    current_hp: current_boss.current_hp.saturating_sub(damage),
                    is_critical: false,
                    damage,
                    target_id,
                };

                sender.send(event).unwrap();
                thread::sleep(std::time::Duration::from_millis(500 + rng.random_range(0..200)));
            }
        });

        self.handle = Some(handle);
    }
}

