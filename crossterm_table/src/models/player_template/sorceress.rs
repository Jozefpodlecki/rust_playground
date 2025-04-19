use super::*;

impl PlayerTemplate {
    pub fn reflux_sorceress() -> PlayerTemplate {
        PlayerTemplate {
            class: Class::Sorceress,
            crit_rate: 0.75,
            crit_damage: 2.0,
            cooldown_reduction: 0.2,
            attack_power: 5e6 as u64,
            skills: vec![
                SkillTemplate {
                    id: 37200,
                    name: "Blaze",
                    priority: 2,
                    min_ratio: 1.0,
                    max_ratio: 2.0,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    cooldown: Duration::seconds(15),
                    buffs: vec![
                        BuffTemplate {
                            category: BuffCategory::Buff,
                            target: BuffTarget::Party,
                            kind: BuffType::DamageAmplification,
                            duration: Duration::seconds(8),
                            value: 0.06
                        }
                    ],
                    ..Default::default()
                },
                SkillTemplate {
                    id: 37260,
                    name: "Esoteric Reaction",
                    priority: 2,
                    min_ratio: 10.0,
                    max_ratio: 20.0,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 37250,
                    name: "Rime Arrow",
                    priority: 2,
                    min_ratio: 10.0,
                    max_ratio: 20.0,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 37280,
                    name: "Reverse Gravity",
                    priority: 2,
                    min_ratio: 1.0,
                    max_ratio: 2.0,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 37320,
                    name: "Seraphic Hail",
                    priority: 2,
                    min_ratio: 10.0,
                    max_ratio: 20.0,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 6,
                    name: "Unknown",
                    priority: 2,
                    min_ratio: 1.0,
                    max_ratio: 2.0,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 7,
                    name: "Unknown",
                    priority: 2,
                    min_ratio: 1.0,
                    max_ratio: 2.0,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 8,
                    name: "Unknown",
                    priority: 2,
                    min_ratio: 1.0,
                    max_ratio: 2.0,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 9,
                    name: "Unknown",
                    priority: 2,
                    min_ratio: 1.0,
                    max_ratio: 2.0,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 10,
                    name: "Unknown",
                    priority: 2,
                    min_ratio: 1.0,
                    max_ratio: 2.0,
                    kind: SkillType::Awakening,
                    cast_duration: Duration::milliseconds(250),
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    }
}