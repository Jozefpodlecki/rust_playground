use std::sync::{Arc, RwLock};

use crate::models::{boss_state::BossState, player_template::BuffType, AttackResult, PlayerTemplate};


pub fn compute_damage(
    player_id: u64,
    skill_id: u32,
    template: PlayerTemplate,
    boss_state: Arc<RwLock<BossState>>
) -> AttackResult {
    // let mut attack_power = self.template.attack_power;
    let mut damage_multiplier = 1.0;

    let mut result = AttackResult::default();
    result.source_id = player_id;
    result.skill_id = skill_id;

    let active_debuffs = boss_state.read().unwrap().active_debuffs;
    for buff in active_debuffs.iter() {
        if buff.kind == BuffType::Brand {
            result.with_brand = true;
            damage_multiplier += 0.1;
        }
    }

    // for (_, buff) in &self.party_state.read().unwrap().active_buffs {
    //     if buff.expires_on > now {
    //         match buff.kind {
    //             BuffType::AttackPowerBuff => {
    //                 result.with_attack_power_buff = true;
    //                 attack_power += buff.value as u64;
    //             },
    //             BuffType::Identity => {
    //                 result.with_identity_buff = true;
    //                 damage_multiplier += 0.1;
    //             },
    //             BuffType::DamageAmplification => {
    //                 damage_multiplier += buff.value;
    //             },
    //             BuffType::HyperAwakeningTechnique => {
    //                 result.with_hat_buff = true;
    //                 damage_multiplier += 0.1;
    //             },
    //             _ => {}
    //         }
    //     }
    // }

    result
}