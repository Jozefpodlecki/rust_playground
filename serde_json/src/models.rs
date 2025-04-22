use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct EncounterTemplate {
    pub name: String,
    pub bosses: Vec<Boss>,
    pub parties: Vec<Party>,
}

#[derive(Debug, Deserialize)]
pub struct Boss {
    pub name: String,
    pub hp: u64,
    pub hp_bars: u32,
    pub minions: Vec<Minion>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Minion {
    pub name: String,
    pub hp: u64,
    pub hp_bars: u32,
    pub conditions: SpawnConditions,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SpawnConditions {
    pub party: u32,

    #[serde(default)]
    pub spawns_when: Vec<SpawnTrigger>,

    #[serde(default)]
    pub spawns_after: Option<String>, // e.g., "30s"
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum SpawnTrigger {
    HpCheck {
        boss: String,
        operator: String,
        hp: u32,
    },
    DeathCheck {
        boss: String,
        state: String,
    },
}

#[derive(Debug, Deserialize)]
pub struct Party {
    pub id: u32,
    pub members: Vec<Member>,
}

#[derive(Debug, Deserialize)]
pub struct Member {
    #[serde(rename = "class")]
    pub class_name: String,
    pub intelligence: u32,
    pub weapon_power: u32,
    pub crit: f32,

    #[serde(default)]
    pub swiftness: u32,
    #[serde(default)]
    pub specialisation: u32,
    #[serde(default)]
    pub buffs: Vec<Buff>,
    #[serde(default)]
    pub skills: Vec<Skill>,
}

#[derive(Debug, Deserialize)]
pub struct Buff {
    pub name: String,
    pub stat: String,
    pub modifier: f32,
}

#[derive(Debug, Deserialize)]
pub struct Skill {
    pub id: u32,
    pub name: String,

    #[serde(default)]
    pub identity_gain: Option<f32>,
    #[serde(default)]
    pub cooldown: Option<u32>,
    #[serde(default)]
    pub requires_full_identity: Option<bool>,
    #[serde(default)]
    pub buffs: Option<Vec<SkillBuff>>,
}

#[derive(Debug, Deserialize)]
pub struct SkillBuff {
    pub name: String,
    pub target: String,
    pub stat: String,
    pub modifier: f32,
    pub duration: u32,
}