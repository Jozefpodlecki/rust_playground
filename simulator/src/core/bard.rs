
use std::{collections::HashMap, f32::consts::E, sync::Arc, thread};

use chrono::{DateTime, Duration, Utc};
use crossbeam::channel::{Receiver, Sender};
use log::info;
use rand::{rng, Rng};

use crate::core::{event::SimulatorEvent, player::*, types::SimulatorContext, utils::{create_bard_skills, create_basic_skills}};

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
#[repr(u32)]
pub enum BardSkills {
    Buff = 100,
    Summon = 101
}

pub struct BardSimulatorPlayer {
    identity_gauge: f64,
    base: Option<SimulatorPlayerBase>,
    handle: Option<thread::JoinHandle<()>>
}

impl BardSimulatorPlayer {
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

        let skills = create_bard_skills(100, attack_power);

        let mut cooldowns = HashMap::new();

        cooldowns.insert(100, DateTime::<Utc>::MIN_UTC);
        cooldowns.insert(101, DateTime::<Utc>::MIN_UTC);

        Self {
            identity_gauge: 0.0,
            handle: None,
            base: Some(SimulatorPlayerBase {
                id: id,
                name,
                class_id: Class::Bard,
                attack_power,
                crit_rate,
                crit_damage,
                cooldown_reduction,
                skills,
                party_id,
                cooldowns,
                ..Default::default()
            }),
        }
    }

    pub fn consume(&mut self) {

    }
}

impl SimulatorPlayer for BardSimulatorPlayer {

    fn base(&self) -> SimulatorPlayerBase {
        self.base.clone().unwrap()
    }

    fn run(&mut self, context: Arc<SimulatorContext>, receiver: Receiver<SimulatorEvent>, sender: Sender<SimulatorEvent>) {
        let mut base = self.base.take().unwrap();

        let handle = thread::spawn(move || {

            info!("{}", base.class_id.as_ref());
            context.barrier.wait();
            let mut rng = rng();

            loop {

                let now = Utc::now();
                let current_boss = context.current_boss.read().unwrap();
                let target_id = current_boss.id;

                if current_boss.current_hp == 0 {
                    break;
                }

                let skill_id = BardSkills::Buff as u32;

                let expires_on = base.cooldowns.get(&skill_id).unwrap();

                if &now < expires_on {
                    thread::sleep(std::time::Duration::from_secs(1));
                    continue;
                }

                let mut ids_to_remove = vec![];

                for (id, summon) in base.summons.iter_mut() {
                    if now > summon.expires_on {
                        ids_to_remove.push(*id);
                    }

                    if summon.next_attack_on > now {
                        continue;
                    }

                    summon.next_attack_on = now + Duration::seconds(1);

                    let damage = 1;

                    if now > summon.next_attack_on {
                        let event = SimulatorEvent::SkillDamage { 
                            source_id: summon.id,
                            skill_id: skill_id,
                            current_hp: current_boss.current_hp.saturating_sub(1),
                            max_hp: current_boss.max_hp,
                            is_critical: false,
                            damage,
                            target_id
                        };

                        sender.send(event).unwrap();

                        for effect in summon.effects.clone() {
                            match effect {
                                SimulatorPlayerSkillEffect::Buff { 
                                    id,
                                    buff_id,
                                    buff_type,
                                    target,
                                    category,
                                    duration
                                } => {
                                    let event = SimulatorEvent::Buff { 
                                        id: buff_id,
                                        buff_type,
                                        source_id: summon.id,
                                        target_id: current_boss.id,
                                        duration
                                    };

                                    sender.send(event).unwrap();
                                },
                                _ => {}
                            }
                        }
                    }
                }

                for id in ids_to_remove {
                    base.summons.remove(&id);
                }

                let skill = base.skills.get(&skill_id).unwrap();

                if skill.deals_damage {
                    let mut damage = base.attack_power as f64;
                    damage = damage * rng.random_range(skill.min_damage..skill.max_damage);

                    let is_critical = rng.random_bool(base.crit_rate);

                    if is_critical {
                        damage = damage * base.crit_damage;
                    }

                    let damage = damage as i64;

                    let event = SimulatorEvent::SkillDamage { 
                        source_id: base.id,
                        skill_id: skill_id,
                        current_hp: current_boss.current_hp.saturating_sub(damage),
                        max_hp: current_boss.max_hp,
                        is_critical,
                        damage,
                        target_id
                    };

                    sender.send(event).unwrap();
                }

                let expires_on = now + skill.cooldown;
                base.cooldowns.insert(skill.id, expires_on);

                for effect in skill.effects.clone() {
                    match effect {
                        SimulatorPlayerSkillEffect::Summon { id, npc_id, duration, effects } => {

                            let summon = base.summons.get(&id);

                            if summon.is_none() {
                                let summon = SimulatorPlayerSummon {
                                    id: rng.random(),
                                    npc_id,
                                    next_attack_on: now + Duration::seconds(1),
                                    expires_on: now + duration,
                                    effects
                                };

                                base.summons.insert(id, summon);
                            }
                            else {
                                let event = SimulatorEvent::Remove { id: id as u64 };
                                sender.send(event).unwrap();
                            }

                        },
                        SimulatorPlayerSkillEffect::Buff { id, buff_id, buff_type, target, category, duration } => {

                            if target == SimulatorPlayerSkillBuffTarget::SelfTarget {
                                let event = SimulatorEvent::Buff {
                                    id: buff_id,
                                    buff_type,
                                    source_id: base.id,
                                    target_id,
                                    duration
                                };

                                sender.send(event).unwrap();
                            }

                            if target == SimulatorPlayerSkillBuffTarget::Party {
                                let members = context.party_map.get(&base.party_id).cloned().unwrap();

                                for target_id in members {
                                    let event = SimulatorEvent::PartyBuff {
                                        id: buff_id,
                                        buff_type,
                                        source_id: base.id,
                                        target_id,
                                        duration
                                    };

                                    sender.send(event).unwrap();
                                }
                            }
                        },
                    }
                }

                thread::sleep(std::time::Duration::from_secs(2));
            }
        });

        self.handle = Some(handle);
    }
}