use chrono::{DateTime, Utc};

use super::Boss;

pub struct BossTemplate {
    pub name: &'static str,
    pub enrage_timer: &'static str,
    pub max_hp: u64,
    pub hp_bars: u64
}

pub struct EncounterTemplate {
    pub name: &'static str,
    pub boss: BossTemplate,
    pub party_count: u64,
}

impl EncounterTemplate {
    pub const ECHIDNA_G1: EncounterTemplate = EncounterTemplate {
        name: "Echidna G1",
        boss: BossTemplate {
            name: "Red Doom Narkiel",
            max_hp: 92.6e9 as u64,
            hp_bars: 180,
            enrage_timer: "15:00"
        },
        party_count: 2,
    };

   pub const ECHIDNA_G2: EncounterTemplate = EncounterTemplate {
        name: "Echidna G2",
        boss: BossTemplate {
            name: "Covetous Master Echidna",
            max_hp: 10.8e10 as u64,
            hp_bars: 285,
            enrage_timer: "09:00",
        },
        party_count: 2,
    };
}