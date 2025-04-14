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
    pub boss: Boss,
    pub enrage_timer: &'static str,
    pub party_count: u64,
}

impl EncounterTemplate {
    pub const ECHIDNA_G1: EncounterTemplate = EncounterTemplate {
        name: "Echidna G1",
        boss: Boss {
            id: 0,
            name: "Narkiel",
            max_hp: 100_000_000_000,
            current_hp: 100_000_000_000,
            hp_percentage: 100.0,
            hp_bars: 300
        },
        enrage_timer: "10:00",
        party_count: 2,
    };

   pub const ECHIDNA_G2: EncounterTemplate = EncounterTemplate {
        name: "Echidna G2",
        boss: Boss {
            id: 0,
            name: "Echidna",
            max_hp: 100_000_000_000,
            current_hp: 100_000_000_000,
            hp_percentage: 100.0,
            hp_bars: 300
        },
        enrage_timer: "10:00",
        party_count: 2,
    };
}