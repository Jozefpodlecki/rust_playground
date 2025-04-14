pub mod encounter_template;
pub mod player_template;

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use strum_macros::{AsRefStr, EnumString};

pub use encounter_template::EncounterTemplate;
pub use player_template::PlayerTemplate;

#[derive(Default, Debug, Copy, Clone, AsRefStr, PartialEq, EnumString)]
#[repr(u32)]
pub enum Class {
    #[default]
    Unknown = 0,
    #[strum(serialize = "Warrior (Male)")]
    WarriorMale = 101,
    Berserker = 102,
    Destroyer = 103,
    Gunlancer = 104,
    Paladin = 105,
    #[strum(serialize = "Warrior (Female)")]
    WarriorFemale = 111,
    Slayer = 112,
    Mage = 201,
    Arcanist = 202,
    Summoner = 203,
    Bard = 204,
    Sorceress = 205,
    #[strum(serialize = "Martial Artist (Female)")]
    MartialArtistFemale = 301,
    Wardancer = 302,
    Scrapper = 303,
    Soulfist = 304,
    Glaivier = 305,
    #[strum(serialize = "Martial Artist (Male)")]
    MartialArtistMale = 311,
    Striker = 312,
    Breaker = 313,
    Assassin = 401,
    Deathblade = 402,
    Shadowhunter = 403,
    Reaper = 404,
    Souleater = 405,
    #[strum(serialize = "Gunner (Male)")]
    GunnerMale = 501,
    Sharpshooter = 502,
    Deadeye = 503,
    Artillerist = 504,
    Machinist = 505,
    #[strum(serialize = "Gunner (Female)")]
    GunnerFemale = 511,
    Gunslinger = 512,
    Specialist = 601,
    Artist = 602,
    Aeromancer = 603,
    Wildsoul = 604,
}

#[derive(Default, Debug, Clone)]
pub struct Party {
    pub id: u64,
    pub players: Vec<Player>,
}

#[derive(Default, Debug, Clone)]
pub struct Encounter {
    pub boss: Boss,
    pub duration: EncounterDuration,
    pub started_on: DateTime<Utc>,
    pub parties: Vec<Party>,
    pub stats: EncounterStats
}

#[derive(Default, Debug, Clone)]
pub struct EncounterStats {
    pub total_damage: u64,
}

#[derive(Default, Debug, Clone)]
pub struct EncounterDuration {
    pub elapsed_seconds: u64,
    pub mmss: String,
}

pub struct BossTemplate {
    pub name: &'static str,
    pub max_hp: u64,
    pub hp_bars: u64
}

#[derive(Default, Debug, Clone)]
pub struct Boss {
    pub id: u64,
    pub name: &'static str,
    pub max_hp: u64,
    pub current_hp: u64,
    pub hp_percentage: f32,
    pub hp_bars: u64
}

pub struct AttackResult {
    pub skill_id: u32,
    pub damage: u64,
    pub is_critical: bool,
}

#[derive(Default, Clone, Debug)]
pub struct Skill {
    pub id: u32,
    pub name: String
}

#[derive(Default, Clone, Debug)]
pub struct Player {
    pub id: u64,
    pub name: String,
    pub class: Class,
    pub stats: PlayerStats,
    pub skills: HashMap<u32, Skill>,
}

#[derive(Default, Clone, Debug)]
pub struct PlayerStats {
    pub crit_rate: f32,
    pub crit_damage: u64,
    pub dps: u64,
    pub total_damage: u64,
    pub damage_with_brand: u64,
    pub damage_with_attack_power_buff: u64,
    pub damage_with_identity_buff: u64,
    pub damage_with_hat_buff: u64,
    pub brand_percentage: f32,
    pub hat_percentage: f32,
    pub attack_power_buff_percentage: f32,
    pub identity_percentage: f32,
    pub skills: PlayerSkillStats
}

#[derive(Default, Clone, Debug)]
pub struct PlayerSkillStats {

}