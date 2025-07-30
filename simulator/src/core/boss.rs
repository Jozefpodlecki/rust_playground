use std::{collections::HashMap, sync::Arc, thread};

use chrono::{DateTime, Duration, Utc};
use crossbeam::channel::{Receiver, Sender};
use log::info;
use rand::{rng, seq::IndexedRandom, RngCore};

use crate::core::{event::SimulatorEvent, player::SimulatorPlayerSkillBuff, types::{EncounterTemplateBoss, EncounterTemplateBossSummonConditon, SimulatorContext}};

pub struct SimulatorBoss {
    pub id: u64,
    pub name: String,
    pub npc_id: u32,
    pub hp_bars: u16,
    pub bar_per_hp: f32,
    pub current_hp: i64,
    pub max_hp: i64,
    next_attack_on: DateTime<Utc>,
    summons: Vec<SimulatorBossSummon>,
    buffs: HashMap<u32, SimulatorPlayerSkillBuff>,
    handle: Option<thread::JoinHandle<()>>,
}

pub struct SimulatorBossSummon {
    pub id: u64,
    pub is_active: bool,
    pub current_hp: i64,
    pub max_hp: i64,
    pub condition: EncounterTemplateBossSummonConditon,
    pub next_attack_on: DateTime<Utc>
}

impl SimulatorBoss {
    pub fn new(args: EncounterTemplateBoss) -> Self {

        let mut rng = rng();

        let mut summons = vec![];

        for summon in args.summons {
            let boss_summon = SimulatorBossSummon {
                id: rng.next_u64(),
                is_active: false,
                max_hp: summon.max_hp,
                current_hp: summon.max_hp,
                condition: summon.condition,
                next_attack_on: DateTime::<Utc>::MIN_UTC
            };

            summons.push(boss_summon);
        }

        Self {
            id: args.id,
            name: "Boss".into(),
            npc_id: args.npc_id,
            current_hp: args.max_hp,
            hp_bars: args.hp_bars,
            bar_per_hp: args.max_hp as f32 / args.hp_bars as f32,
            max_hp: args.max_hp,
            summons,
            buffs: HashMap::new(),
            next_attack_on: DateTime::<Utc>::MIN_UTC,
            handle: None,
        }
    }

    pub fn run(&mut self, context: Arc<SimulatorContext>, receiver: Receiver<SimulatorEvent>, sender: Sender<SimulatorEvent>) {
        let context = context.clone();

        let handle = thread::spawn(move || Self::run_inner(context, receiver, sender));

        self.handle = Some(handle);
    }

    #[inline(always)]
    pub fn run_inner(context: Arc<SimulatorContext>, receiver: Receiver<SimulatorEvent>, sender: Sender<SimulatorEvent>) {

        context.barrier.wait();
        
        loop {
            Self::tick(&context, &receiver, &sender);
        }
    }

    #[inline(always)]
    pub fn tick(context: &Arc<SimulatorContext>, receiver: &Receiver<SimulatorEvent>, sender: &Sender<SimulatorEvent>) {
        let id = context.current_boss.read().unwrap().id;
        let player_ids = context.player_ids.clone();
        let mut rng = rng();
        let now = Utc::now();
        let hp_bars;
        let duration = std::time::Duration::from_secs(1);

        if let Ok(event) = receiver.recv_timeout(duration) {
            match event {
                SimulatorEvent::Buff { id, buff_type, source_id, target_id, duration } => {
                    let mut context = context.current_boss.write().unwrap();

                    if context.buffs.get(&id).is_some() {
                        let event = SimulatorEvent::RemoveBuff { id };
                        sender.send(event).unwrap();
                    }

                    let buff =  SimulatorPlayerSkillBuff {
                        id,
                        buff_type,
                        expires_on: now + duration
                    };

                    context.buffs.insert(id, buff);
                },
                SimulatorEvent::SkillDamage {
                    damage,
                    target_id,
                    .. } => {
                    if target_id == id {
                        let mut context = context.current_boss.write().unwrap();

                        if context.current_hp == 0 {
                            return;
                        }

                        if context.current_hp > damage {
                            info!("boss hp");
                            context.current_hp -= damage;
                        }
                        else {
                            context.current_hp = 0;
                        }

                        hp_bars = context.current_hp as f32 * context.bar_per_hp;
                        context.hp_bars = hp_bars as u16;
                    }

                    let mut context = context.current_boss.write().unwrap();
                    for summon in context.summons.iter_mut() {
                        if summon.id == target_id && summon.is_active {
                            summon.current_hp = (summon.current_hp - damage).max(0);
                            
                            if summon.current_hp == 0 {
                                summon.is_active = false;

                                let event = SimulatorEvent::EntityDied { id: summon.id };
                                sender.send(event).unwrap();
                            }
                            break;
                        }
                    }
                },
                _ => {}
            }
        }

        {
            let (hp_bars, current_hp) = {
                let context = context.current_boss.read().unwrap();
                (context.hp_bars, context.current_hp)
            };
            let mut context = context.current_boss.write().unwrap();

            for summon in context.summons.iter_mut() {
                match (summon.is_active, summon.condition) {
                    (true, _) => {
                        if now > summon.next_attack_on {
                            let target_id = *player_ids.choose(&mut rng).unwrap();
                            let event = SimulatorEvent::SkillDamage {
                                source_id: summon.id,
                                skill_id: 10001,
                                current_hp: summon.current_hp,
                                max_hp: summon.max_hp,
                                is_critical: false,
                                damage: 1,
                                target_id 
                            };

                            sender.send(event).unwrap();
                        }
                    },
                    (false, EncounterTemplateBossSummonConditon::HpBars(bars)) => {
                        if hp_bars <= bars {
                            summon.is_active = true;
                        }
                    },
                    (false, EncounterTemplateBossSummonConditon::Death) => {
                        if current_hp == 0 {
                            summon.is_active = true;
                        }
                    },
                }
            }
        }

        {
            let context = context.current_boss.read().unwrap();
        
            if context.current_hp == 0 {
                return;
            }

            if context.next_attack_on > now {
                return;
            }
        }

        {
            let mut context = context.current_boss.write().unwrap();
            context.next_attack_on = now + Duration::seconds(1);
        }

        let target_id = *player_ids.choose(&mut rng).unwrap();

        let event = SimulatorEvent::SkillDamage { 
            is_critical: false,
            damage: 1,
            skill_id: 100000,
            current_hp: 0,
            max_hp: 0,
            source_id: id,
            target_id,
        };

        sender.send(event).unwrap();
    }
}