use std::{
    sync::{atomic::{AtomicBool, Ordering}, mpsc, Arc, Mutex, RwLock},
    thread::{self, sleep},
};
use chrono::{DateTime, Utc};
use rand::{Rng, rng};
use std::collections::HashMap;

use crate::{models::{player_template::*, *}, utils::{format_duration, random_number_in_range}};

pub fn update_party_stats(
    duration_seconds: i64,
    party_stats: &mut PartyStats,
    attack_result: &AttackResult
) {
    if attack_result.damage == 0 {
        return;
    }

    party_stats.total_damage += attack_result.damage;

    if duration_seconds == 0 {
        party_stats.dps = 0;
    }
    else {
        party_stats.dps = party_stats.total_damage / duration_seconds as u64;
    }
}

pub fn update_encounter_stats(
    duration_seconds: i64,
    boss: &mut Boss,
    encounter_stats: &mut EncounterStats,
    attack_result: &AttackResult) {

    if attack_result.damage == 0 {
        return;
    }

    if attack_result.damage >= boss.current_hp {
        boss.current_hp = 0;
        let damage = attack_result.damage - boss.current_hp;
        encounter_stats.total_damage += damage;
        encounter_stats.ttk = "00:00".to_string();
    }
    else {
        boss.current_hp = boss.current_hp - attack_result.damage;
        boss.hp_percentage = boss.current_hp as f32 / boss.max_hp as f32;
        boss.hp_bars = boss.current_hp as f32 / boss.bar_per_hp;
        encounter_stats.total_damage += attack_result.damage;
        
        if duration_seconds == 0 {
            encounter_stats.dps = 0;
            encounter_stats.ttk = "INF".to_string();
        }
        else {
            encounter_stats.dps = encounter_stats.total_damage / duration_seconds as u64;   
            let ttk_seconds = boss.current_hp / encounter_stats.dps;
            encounter_stats.ttk = format_duration(ttk_seconds as i64);
        }
    }
}

pub fn update_player_stats(
    player_stats: &mut PlayerStats,
    duration_seconds: i64,
    attack_result: &AttackResult) {

    if attack_result.damage == 0 {
        return;
    }

    player_stats.total_damage += attack_result.damage;

    if duration_seconds != 0 {
        player_stats.dps = (player_stats.total_damage as f32 / duration_seconds as f32) as u64;
    }

    player_stats.crit_damage += if attack_result.is_critical {
        attack_result.damage
    } else {
        0
    };
    
    player_stats.crit_rate = player_stats.crit_damage as f32 / player_stats.total_damage as f32;

    if attack_result.with_brand {
        player_stats.damage_with_brand += attack_result.damage;
        player_stats.brand_percentage = player_stats.damage_with_brand as f32 / player_stats.total_damage as f32;
    }

    if attack_result.with_attack_power_buff {
        player_stats.damage_with_attack_power_buff += attack_result.damage;
        player_stats.attack_power_buff_percentage = player_stats.damage_with_attack_power_buff as f32 / player_stats.total_damage as f32;
    }

    if attack_result.with_identity_buff {
        player_stats.damage_with_identity_buff += attack_result.damage;
        player_stats.identity_percentage = player_stats.damage_with_identity_buff as f32 / player_stats.total_damage as f32;
    }

    if attack_result.with_hat_buff {
        player_stats.damage_with_hat_buff += attack_result.damage;
        player_stats.hat_percentage = player_stats.damage_with_hat_buff as f32 / player_stats.total_damage as f32;
    }

    let skill_stats = player_stats.skills.skill.entry(attack_result.skill_id).or_default();

    skill_stats.hit_count += 1;

    if attack_result.is_critical {
        player_stats.skills.crit_count += 1;
        skill_stats.crit_count += 1;
    }

    if attack_result.is_hyper_awakening {
        player_stats.hyper_awakening_damage += attack_result.damage;
    }

    if attack_result.hit_option == HitOption::Back {
        skill_stats.back_attack.count += 1;
        skill_stats.back_attack.damage += attack_result.damage;
    }

    if attack_result.hit_option == HitOption::Frontal {
        skill_stats.front_attack.count += 1;
        skill_stats.front_attack.damage += attack_result.damage;
    }
}