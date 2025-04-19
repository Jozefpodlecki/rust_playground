use std::{collections::HashMap, sync::{atomic::{AtomicBool, Ordering}, mpsc::Sender, Arc, RwLock}, thread::{self, sleep}};

use chrono::{DateTime, Utc};
use rand::{rng, Rng};

use crate::{models::{player_template::{BuffType, SkillType}, AttackResult, Buff, PartyState, PlayerTemplate}, multi_thread_simulator::{apply_buffs::apply_buffs, attack::{get_available_skills, perform_attack}, id_generator::{self, IdGenerator}}};

use super::Worker;

pub struct BardWorker {
    id_generator: IdGenerator,
    skill_cooldowns: HashMap<u32, DateTime<Utc>>,
    active_buffs: HashMap<u32, Buff>,
    active_buff_types: HashMap<BuffType, DateTime<Utc>>,
    identity: f32,
    party_state: Arc<RwLock<PartyState>>,
    boss_state: Arc<RwLock<BossState>>,
    template: PlayerTemplate,
    started_on: DateTime<Utc>,
    player_id: u64,
    tx: Sender<AttackResult>,
    control_flag: Arc<AtomicBool>,
}

impl BardWorker {
    pub fn new(
        template: PlayerTemplate,
        party_state: Arc<RwLock<PartyState>>,
        boss_state: Arc<RwLock<BossState>>,
        started_on: DateTime<Utc>,
        player_id: u64,
        tx: Sender<AttackResult>,
        control_flag: Arc<AtomicBool>,
    ) -> Self {
        Self {
            id_generator: IdGenerator::new(),
            skill_cooldowns: HashMap::new(),
            active_buffs: HashMap::new(),
            active_buff_types: HashMap::new(),
            identity: 0.0,
            party_state,
            boss_state,
            template,
            started_on,
            player_id,
            tx,
            control_flag,
        }
    }
}

impl Worker for BardWorker {
    fn start_loop(&mut self) {

        while !self.control_flag.load(Ordering::Acquire) {
            thread::yield_now();
        }

        while self.boss_state.read().unwrap().current_hp > 0 {
            let now: DateTime<Utc> = Utc::now();

            self.skill_cooldowns.retain(|_, &mut time| time > now);

            if !self.control_flag.load(Ordering::Relaxed) {
                break;
            }

            if let Some(result) = self.perform_attack() {
                self.tx.send(result).unwrap();
            }

            if self.skill_cooldowns.len() >= 8 {
                if let Some(next_cooldown) = self.skill_cooldowns.values().min() {
                    let duration = (*next_cooldown - now).to_std().unwrap_or_default();
                    sleep(duration);
                }
            }
        }
    }
}

impl BardWorker {

    pub fn perform_attack(&mut self) -> Option<AttackResult> {
        
        let now = Utc::now();
        let duration_seconds = (now - self.started_on).num_seconds();

        let sorted_skills = get_available_skills(
            now,
            &self.template.skills,
            &self.skill_cooldowns);

        for skill_template in sorted_skills {
            if skill_template.kind == SkillType::HyperAwakening && duration_seconds < 180 {
                continue;
            }

            if skill_template.requires_identity && self.identity < 2.0 {
                continue;
            }

            if skill_template.buffs.iter().any(|buff| {
                self.active_buff_types
                    .get(&buff.kind)
                    .map_or(false, |&expires_on| expires_on > now)
            }
                && !skill_template.buffs.iter().any(|buff| buff.kind == BuffType::Brand)) {
                continue;
            }

            let expires_on = now + skill_template.cooldown;
            self.skill_cooldowns
                .insert(skill_template.id, expires_on);

            sleep(skill_template.cast_duration.to_std().unwrap());

            self.identity += skill_template.identity_gain;

           
        
            apply_buffs(
                &mut self.id_generator,
                &self.template,
                &skill_template.buffs,
                now,
                self.party_state.clone(),
                self.boss_state.clone(),
                &mut self.active_buffs,
                &mut self.active_buff_types);

           

       

           

            let mut damage = 0f32;

            if skill_template.min_ratio != 0.0 {
                let min = attack_power as f32 * skill_template.min_ratio * damage_multiplier;
                let max = attack_power as f32 * skill_template.max_ratio * damage_multiplier;
                damage = self.id_generator.next_f32(min..max);
            }

            let is_critical = self.id_generator.next_bool(self.template.crit_rate as f64);

            result.damage = if is_critical { (damage * self.template.crit_damage) as u64 } else { damage as u64 };
            result.is_critical = is_critical;

            return Some(result);
        }

        None
    }
}