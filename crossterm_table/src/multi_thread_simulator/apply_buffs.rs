use std::{process::id, sync::{Arc, RwLock}};

use chrono::{DateTime, Utc};

use super::*;

pub fn apply_buffs(
    id_generator: &mut IdGenerator,
    player_template: &PlayerTemplate,
    buffs: &Vec<BuffTemplate>,
    now: DateTime<Utc>,
    party_state: Arc<RwLock<PartyState>>,
    boss_state: Arc<RwLock<BossState>>,
    active_buffs: &mut HashMap<u32, Buff>) {

    for buff_template in buffs {

        let expires_on = now + buff_template.duration;
        let mut buff = Buff {
            target: buff_template.target,
            kind: buff_template.kind,
            expires_on,
            value: 0.0,
        };

        if buff_template.kind == BuffType::DamageAmplification {
            buff.value = buff_template.value;
        }

        if buff_template.category == BuffCategory::Debuff {
            let mut instance_id = random_number_in_range(1000..9999);
            let mut boss_state = boss_state.write().unwrap();

            while boss_state.active_debuffs.contains_key(&instance_id) {
                instance_id = random_number_in_range(1000..9999);
            }

            boss_state.active_debuffs.insert(instance_id, buff.clone());
        }

        if buff_template.category == BuffCategory::Buff {

            if buff_template.target == BuffTarget::TargetSelf {
                apply_self_buff(now, buff_template, id_generator, active_buffs);
            }

            if buff_template.target == BuffTarget::Party {
                apply_attack_power_buff(
                    now,
                    player_template,
                    buff_template,
                    id_generator,
                    &party_state);
            }
        }
    }
}

pub fn apply_self_buff(
    now: DateTime<Utc>,
    buff_template: &BuffTemplate,
    id_generator: &mut IdGenerator,
    active_buffs: &mut HashMap<u32, Buff>) {
    let expires_on = now + buff_template.duration;
    let mut buff = Buff {
        target: buff_template.target,
        kind: buff_template.kind,
        expires_on,
        value: 0.0,
    };
    let instance_id = id_generator.next_buff_instance_id();

    active_buffs.insert(
        instance_id,
        buff.clone()
    );   
}

pub fn apply_attack_power_buff(
    now: DateTime<Utc>,
    player_template: &PlayerTemplate,
    buff_template: &BuffTemplate,
    id_generator: &mut IdGenerator,
    party_state: &Arc<RwLock<PartyState>>) {
    let expires_on = now + buff_template.duration;
    let mut buff = Buff {
        target: buff_template.target,
        kind: buff_template.kind,
        expires_on,
        value: 0.0,
    };

    if buff_template.kind == BuffType::AttackPowerBuff {
        let buff_attack_power = (player_template.attack_power as f32 * 0.15) as u64;
        buff.value = buff_attack_power as f32;
    }

    let instance_id = id_generator.next_buff_instance_id();
    let mut party_state = party_state.write().unwrap();

    party_state.active_buffs.insert(
        instance_id,
        buff
    );   
}