use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, sleep, JoinHandle};
use anyhow::*;
use chrono::{DateTime, Duration, Utc};
use log::{debug, warn};
use multiqueue::{BroadcastReceiver, BroadcastSender};

use crate::models::*;

use rand::Rng;

pub struct DpsThread {
    stats: PlayerStats,
    handle: Option<JoinHandle<Result<()>>>,
    close_flag: Arc<AtomicBool>
}

impl DpsThread {
    pub fn new(stats: PlayerStats) -> Self {
        Self {
            stats,
            handle: None,
            close_flag: Arc::new(AtomicBool::new(false))
        }
    }

    pub fn start(&mut self, tx: BroadcastSender<Message>, rx: BroadcastReceiver<Message>) {
        let stats = self.stats.clone();
        let start_time = Utc::now();
        let close_flag = self.close_flag.clone();

        let handle = thread::spawn(move || {
            let mut cooldowns: HashMap<u32, DateTime<Utc>> = HashMap::new();
            let mut active_buffs: HashMap<u32, DateTime<Utc>> = HashMap::new();

            let skill_instances: Vec<SkillInstance> = stats.skills.iter().map(|skill| {
                let effective_cooldown = Self::apply_cooldown_reduction(stats.cooldown_reduction, skill);
                let has_quick_recharge = skill.id == 45300;
                let priority = Self::calculate_priority(skill);

                SkillInstance {
                    skill,
                    effective_cooldown,
                    has_quick_recharge,
                    priority
                }
            }).collect();

            loop {
                if close_flag.load(Ordering::Relaxed) {
                    debug!("Stopping");
                    break;
                }

                let now = Utc::now();
                let elapsed = now - start_time;
                cooldowns.retain(|_, expires_on| *expires_on > now);
                active_buffs.retain(|_, expires_on| *expires_on > now);

                let skill = Self::get_available_skill(
                    now,
                    elapsed,
                    active_buffs.len() > 0,
                    &mut cooldowns,
                    &skill_instances);

                if let Some(skill_instance) = skill {
                    let skill_name = &skill_instance.skill.name;
                    let expires_on = now + skill_instance.effective_cooldown;

                    debug!("Using skill: {}, cooldown: {}s", skill_name, skill_instance.effective_cooldown.num_seconds());
                    cooldowns.insert(skill_instance.skill.id, expires_on);
                    sleep(skill_instance.skill.cast_time.to_std()?);

                    if skill_instance.has_quick_recharge {
                        Self::try_apply_quick_recharge(&mut cooldowns, now);
                    }

                    for effect in &skill_instance.skill.effects {
                        if let Some(buff) = &effect.buff {
                            let expires_on = now + buff.duration;
                            debug!("Applying buff: {}, duration: {}s", buff.id, buff.duration.num_seconds());
                            active_buffs.insert(buff.id, expires_on);

                            tx.try_send(Message::Damage { source_id: 1 })?;
                        }
                    }
                }

                if let Some((skill_id, duration)) = Self::get_shortest_cooldown(now, &mut cooldowns) {
                    debug!( "Next skill to use: id {} with {}s remaining cooldown. Sleeping until ready.", skill_id, duration.num_seconds());
                    sleep(duration.to_std()?);
                }
            }

            Ok(())
        });

        self.handle = Some(handle);
    }

    fn get_shortest_cooldown(now: DateTime<Utc>, cooldowns: &mut HashMap<u32, DateTime<Utc>>) -> Option<(u32, Duration)> {        
        if cooldowns.len() < 7 {
            return None;
        }

        cooldowns.iter()
            .filter_map(|(&id, &cooldown_expiration)| {
                let remaining_cooldown = cooldown_expiration.signed_duration_since(now);
                if remaining_cooldown > chrono::Duration::zero() {
                    Some((id, remaining_cooldown))
                } else {
                    None
                }
            })
            .min_by_key(|&(_, remaining_cooldown)| remaining_cooldown)
    }

    fn get_available_skill<'a>(
        now: DateTime<Utc>,
        elapsed: Duration,
        has_active_self_buff: bool,
        cooldowns: &mut HashMap<u32, DateTime<Utc>>,
        skill_instances: &'a [SkillInstance<'a>],
    ) -> Option<&'a SkillInstance<'a>> {
        let mut available_skills: Vec<&SkillInstance> = skill_instances.iter()
            .filter(|skill_instance| {
                let on_cooldown = cooldowns.get(&skill_instance.skill.id).map_or(false, |&expires_on| now < expires_on);
                
                if on_cooldown {
                    return false;
                }

                skill_instance.can_use(elapsed, has_active_self_buff)
            })
            .collect();

        available_skills.sort_by_key(|skill_instance| skill_instance.priority);

        available_skills.first().cloned()
    }

    fn calculate_priority(skill: &Skill) -> u8 {
        match skill.name {
            "Wild Stomp" => 1,
            "Ragna Deathblade" => 2,
            "Brutal Impact" => 3,
            "Volcanic Eruption" => 4,
            "Spiral Deathblade" => 5,
            "Guillotine" => 6,
            "Fatal Sword" => 7,
            "Punishing Draw" => 8,
            "Furious Claw" => 9,
            _ => 10,
        }
    }

    fn apply_cooldown_reduction(cooldown_reduction: f32, skill: &Skill) -> Duration {
        let cooldown = skill.cooldown;
        let effective_cooldown_secs = cooldown.num_seconds() as f32 * (1.0 - cooldown_reduction);
        Duration::seconds(effective_cooldown_secs as i64)
    }

    fn try_apply_quick_recharge(
        cooldowns: &mut HashMap<u32, DateTime<Utc>>,
        now: DateTime<Utc>,
    ) {
        let mut rng = rand::rng();
        let reset_chance: f32 = rng.random();

        if reset_chance > 0.1 {
            return;
        }

        debug!("Quick Recharge triggered! Reducing cooldown by 16%");
        for (_, cooldown) in cooldowns {
            
            let value = *cooldown;
            let duration = value - now;
            
            if duration <= Duration::zero() {
                warn!("{:?} {:?} {:?}", duration, cooldown, now);
                *cooldown = now;
                continue;
            }

            let duration_millis = duration.num_milliseconds() as f32 * 0.16;
            let duration = Duration::milliseconds(duration_millis as i64);
            *cooldown = value - duration;
        }
    }

    pub fn wait(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap().unwrap();
        }
    }

    pub fn stop(&mut self) {
        self.close_flag.store(true, Ordering::Relaxed);
        self.wait();
    }
}


// for (id, expires_on) in &cooldowns {
//     let duration = *expires_on - now;
//     debug!("skill: {}, cooldown: {}s", id, duration.num_seconds());
// }
