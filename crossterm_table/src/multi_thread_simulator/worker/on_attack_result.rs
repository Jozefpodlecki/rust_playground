use crate::utils::format_duration;

use super::*;

pub fn on_attack_result(
    started_on: DateTime<Utc>,
    encounter: &mut Encounter,
    boss_state: &Arc<RwLock<BossState>>,
    attack_result: AttackResult) {
        
    let now = Utc::now();
    let elapsed_duration = now - started_on;
    let duration_seconds = elapsed_duration.num_seconds();

    if attack_result.damage > 0 {
        let mut boss_state = boss_state.write().unwrap();
        
        if boss_state.current_hp > attack_result.damage  {
            boss_state.current_hp -= attack_result.damage;
        }
        else {
            boss_state.current_hp = 0;
        }
    }

    let Encounter {
        boss,
        parties,
        stats: encounter_stats,
        duration,
        ..
    } = encounter;

    duration.elapsed_seconds = duration_seconds;
    duration.mmss = format_duration(duration_seconds);

    for party in parties.iter_mut() {
        for player in party.players.iter_mut() {

            if attack_result.source_id == player.id {
                update_player_stats(
                    &mut player.stats,
                    duration_seconds,
                    &attack_result);

                update_encounter_stats(
                    duration_seconds,
                    boss,
                    encounter_stats,
                    &attack_result);
                update_party_stats(
                    duration_seconds,
                    &mut party.stats,
                    &attack_result,
                    encounter_stats.total_damage);
            }
        }
    }

    for party in &mut encounter.parties {
        party.players.sort_by(|a, b| b.stats.dps.cmp(&a.stats.dps));
    }
}