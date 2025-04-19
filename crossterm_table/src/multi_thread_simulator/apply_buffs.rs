use std::{process::id, sync::{Arc, RwLock}};

use chrono::{DateTime, Utc};

use super::*;

pub fn apply_buffs(
    id_generator: &mut IdGenerator,
    player_template: &PlayerTemplate,
    buffs: &Vec<BuffTemplate>,
    now: DateTime<Utc>,
    boss_state: Arc<RwLock<BossState>>,
    active_buffs: &mut HashMap<u32, Buff>,
    active_buff_types: &mut HashMap<BuffType, DateTime<Utc>>,
) {

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
            let mut instance_id = id_generator.next_buff_instance_id();
            let mut boss_state = boss_state.write().unwrap();

            while boss_state.active_debuffs.contains_key(&instance_id) {
                instance_id = id_generator.next_buff_instance_id();
            }

            boss_state.active_debuffs.insert(instance_id, buff.clone());
        }

        if buff_template.category == BuffCategory::Buff {

            if buff_template.target == BuffTarget::TargetSelf {
                apply_self_buff(
                    now,
                    buff_template,
                    id_generator,
                    active_buffs);
            }

            if buff_template.target == BuffTarget::Party {
                if buff_template.kind == BuffType::AttackPowerBuff {
                    apply_attack_power_buff(
                        now,
                        player_template,
                        buff_template,
                        id_generator,
                        &party_state,
                        active_buff_types);
                }

                if buff_template.kind == BuffType::Identity {
                    apply_identity_buff(
                        now,
                        player_template,
                        buff_template,
                        id_generator,
                        &party_state,
                        active_buff_types);
                }

                if buff_template.kind == BuffType::Brand {
                    apply_brand_buff(
                        now,
                        player_template,
                        buff_template,
                        id_generator,
                        &party_state,
                        active_buff_types);
                }
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

pub fn apply_identity_buff(
    now: DateTime<Utc>,
    player_template: &PlayerTemplate,
    buff_template: &BuffTemplate,
    id_generator: &mut IdGenerator,
    party_state: &Arc<RwLock<PartyState>>,
    active_buff_types: &mut HashMap<BuffType, DateTime<Utc>>) {
    let expires_on = now + buff_template.duration;
    let mut buff = Buff {
        target: buff_template.target,
        kind: buff_template.kind,
        expires_on,
        value: 0.0,
    };

    let buff_attack_power = (player_template.attack_power as f32 * 0.15) as u64;
    buff.value = buff_attack_power as f32;

    let instance_id = id_generator.next_buff_instance_id();
    let mut party_state = party_state.write().unwrap();

    active_buff_types.insert(buff.kind, expires_on);

    party_state.active_buffs.insert(
        instance_id,
        buff
    );   
}

pub fn apply_brand_buff(
    now: DateTime<Utc>,
    player_template: &PlayerTemplate,
    buff_template: &BuffTemplate,
    id_generator: &mut IdGenerator,
    party_state: &Arc<RwLock<PartyState>>,
    active_buff_types: &mut HashMap<BuffType, DateTime<Utc>>) {
    let expires_on = now + buff_template.duration;
    let mut buff = Buff {
        target: buff_template.target,
        kind: buff_template.kind,
        expires_on,
        value: 0.0,
    };

    let instance_id = id_generator.next_buff_instance_id();
    let mut party_state = party_state.write().unwrap();

    if let Some((instance_id, existing_buff)) = party_state
        .active_buffs
        .iter_mut()
        .find(|(_, buff)| buff.kind == buff_template.kind)
    {
        existing_buff.expires_on = expires_on;
        existing_buff.value = 0.0;
        active_buff_types.insert(buff.kind, expires_on);
    } else {
        active_buff_types.insert(buff.kind, expires_on);
        party_state.active_buffs.insert(instance_id, buff);
}
}

pub fn apply_attack_power_buff(
    now: DateTime<Utc>,
    player_template: &PlayerTemplate,
    buff_template: &BuffTemplate,
    id_generator: &mut IdGenerator,
    party_state: &Arc<RwLock<PartyState>>,
    active_buff_types: &mut HashMap<BuffType, DateTime<Utc>>) {
    let expires_on = now + buff_template.duration;
    let mut buff = Buff {
        target: buff_template.target,
        kind: buff_template.kind,
        expires_on,
        value: 0.0,
    };

    let buff_attack_power = (player_template.attack_power as f32 * 0.15) as u64;
    buff.value = buff_attack_power as f32;

    let instance_id = id_generator.next_buff_instance_id();
    let mut party_state = party_state.write().unwrap();

    active_buff_types.insert(buff.kind, expires_on);

    party_state.active_buffs.insert(
        instance_id,
        buff
    );   
}