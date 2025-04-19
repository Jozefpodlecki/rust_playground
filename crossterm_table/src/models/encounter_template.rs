use chrono::{DateTime, Utc};

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

    pub const BEHEMOTH_G1: EncounterTemplate = EncounterTemplate {
        name: "Behemoth G1",
        boss: BossTemplate {
            name: "Behemoth, the Storm Commander",
            max_hp: 2.08e11 as u64,
            hp_bars: 500,
            enrage_timer: "12:00",
        },
        party_count: 4,
    };

    pub const BEHEMOTH_G2: EncounterTemplate = EncounterTemplate {
        name: "Behemoth G2",
        boss: BossTemplate {
            name: "Behemoth, Cruel Storm Slayer",
            max_hp: 2.93e11 as u64,
            hp_bars: 705,
            enrage_timer: "12:00",
        },
        party_count: 4,
    };
}