use chrono::{DateTime, Utc};
use rand::{rng, Rng};
use std::collections::HashSet;

use crate::core::{player::Class, types::*};

pub fn template_3dps_1support() -> EncounterTemplate {
    let rng = rng();
    let mut used_u32_ids = HashSet::new();
    let mut used_u64_ids = HashSet::new();

    let mut next_u32 = || {
        let mut rng = rng.clone();
        loop {
            let id: u32 = rng.random();
            if used_u32_ids.insert(id) {
                return id;
            }
        }
    };

    let mut next_u64 = || {
        let mut rng = rng.clone();
        loop {
            let id: u64 = rng.random();
            if used_u64_ids.insert(id) {
                return id;
            }
        }
    };

    let template = EncounterTemplate {
        boss: EncounterTemplateBoss {
            id: next_u64(),
            npc_id: 1,
            max_hp: 1e9 as i64,
            hp_bars: 100,
            summons: vec![
                EncounterTemplateBossSummon {
                    id: next_u64(),
                    npc_id: 1,
                    max_hp: 1e5 as i64,
                    hp_bars: 1,
                    condition: EncounterTemplateBossSummonConditon::HpBars(50)
                }
            ]
        },
        parties: vec![EncounterTemplateParty {
            id: next_u32(),
            members: vec![
                EncounterTemplatePartyMember {
                    id: next_u64(),
                    name: Class::Sorceress.as_ref().to_string(),
                    class_id: Class::Sorceress,
                    attack_power: 1e6 as i64,
                    cooldown_reduction: 0.2,
                    crit_rate: 0.75,
                    crit_damage: 2.0
                },
                EncounterTemplatePartyMember {
                    id: next_u64(),
                    name: Class::Gunslinger.as_ref().to_string(),
                    class_id: Class::Gunslinger,
                    attack_power: 1e6 as i64,
                    cooldown_reduction: 0.2,
                    crit_rate: 0.15,
                    crit_damage: 2.0
                },
                EncounterTemplatePartyMember {
                    id: next_u64(),
                    name: Class::Berserk.as_ref().to_string(),
                    class_id: Class::Berserk,
                    attack_power: 1e6 as i64,
                    cooldown_reduction: 0.2,
                    crit_rate: 0.75,
                    crit_damage: 2.0
                },
                EncounterTemplatePartyMember {
                    id: next_u64(),
                    name: Class::Bard.as_ref().to_string(),
                    class_id: Class::Bard,
                    attack_power: 1e6 as i64,
                    cooldown_reduction: 0.6,
                    crit_rate: 0.1,
                    crit_damage: 2.0
                },
            ],
        }],
    };

    template
}