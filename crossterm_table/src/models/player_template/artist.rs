use crate::models::*;

use super::*;

pub fn get_artist_skills() -> Vec<SkillTemplate> {
    vec![
        SkillTemplate {
            id: 31230,
            name: "Paint: Ink Well",
            priority: 3,
            min_ratio: 0.1,
            max_ratio: 0.2,
            identity_gain: 0.2,
            kind: SkillType::Normal,
            cast_duration: Duration::milliseconds(250),
            cooldown: Duration::seconds(24),
            ..Default::default()
        },
        SkillTemplate {
            id: 31210,
            name: "Stroke: Hopper",
            priority: 4,
            min_ratio: 0.1,
            max_ratio: 0.2,
            identity_gain: 0.3,
            kind: SkillType::Normal,
            cast_duration: Duration::milliseconds(250),
            cooldown: Duration::seconds(16),
            ..Default::default()
        },
        SkillTemplate {
            id: 31420,
            name: "Paint: Drawing Orchids",
            priority: 5,
            min_ratio: 0.1,
            max_ratio: 0.2,
            identity_gain: 0.1,
            kind: SkillType::Normal,
            cast_duration: Duration::milliseconds(250),
            buffs: vec![
                BuffTemplate {
                    category: BuffCategory::Buff,
                    target: BuffTarget::Party,
                    kind: BuffType::Brand,
                    duration: Duration::seconds(12),
                    value: 0.1
                }
            ],
            cooldown: Duration::seconds(24),
            ..Default::default()
        },
        SkillTemplate {
            id: 31450,
            name: "Paint: Starry Night",
            priority: 6,
            min_ratio: 0.1,
            max_ratio: 0.2,
            identity_gain: 0.2,
            kind: SkillType::Normal,
            cast_duration: Duration::milliseconds(250),
            cooldown: Duration::seconds(15),
            ..Default::default()
        },
        SkillTemplate {
            id: 31220,
            name: "Paint: Illusion Door",
            priority: 7,
            min_ratio: 0.1,
            max_ratio: 0.2,
            identity_gain: 0.3,
            kind: SkillType::Normal,
            cast_duration: Duration::milliseconds(250),
            cooldown: Duration::seconds(36),
            ..Default::default()
        },
        SkillTemplate {
            id: 31410,
            name: "Paint: Sun Well",
            priority: 8,
            min_ratio: 0.1,
            max_ratio: 0.2,
            identity_gain: 0.1,
            kind: SkillType::Normal,
            cast_duration: Duration::milliseconds(250),
            buffs: vec![
                BuffTemplate {
                    category: BuffCategory::Buff,
                    target: BuffTarget::Party,
                    kind: BuffType::AttackPowerBuff,
                    duration: Duration::seconds(7),
                    value: 0.0
                }
            ],
            cooldown: Duration::seconds(15),
            ..Default::default()
        },
        SkillTemplate {
            id: 31490,
            name: "Paint: Pouncing Tiger",
            priority: 9,
            min_ratio: 0.1,
            max_ratio: 0.2,
            identity_gain: 0.1,
            kind: SkillType::Brand,
            cast_duration: Duration::milliseconds(250),
            cooldown: Duration::seconds(15),
            ..Default::default()
        },
        SkillTemplate {
            id: 31400,
            name: "Paint: Sunsketch",
            priority: 10,
            min_ratio: 0.1,
            max_ratio: 0.2,
            identity_gain: 0.1,
            kind: SkillType::AttackPowerBuff,
            cast_duration: Duration::milliseconds(250),
            buffs: vec![
                BuffTemplate {
                    category: BuffCategory::Buff,
                    target: BuffTarget::Party,
                    kind: BuffType::AttackPowerBuff,
                    duration: Duration::seconds(7),
                    value: 0.0
                }
            ],
            cooldown: Duration::seconds(27),
            ..Default::default()
        },
        SkillTemplate {
            id: 31950,
            name: "Paint: Dragon Engraving",
            priority: 11,
            min_ratio: 0.1,
            max_ratio: 0.2,
            identity_gain: 0.1,
            kind: SkillType::Normal,
            cast_duration: Duration::milliseconds(250),
            buffs: vec![
                BuffTemplate {
                    category: BuffCategory::Buff,
                    target: BuffTarget::Party,
                    kind: BuffType::HyperAwakeningTechnique,
                    duration: Duration::seconds(24),
                    value: 0.0
                },
                BuffTemplate {
                    category: BuffCategory::Buff,
                    target: BuffTarget::Party,
                    kind: BuffType::HyperAwakeningTechniqueOutgoingDamage,
                    duration: Duration::seconds(12),
                    value: 0.0
                }
            ],
            cooldown: Duration::seconds(72),
            ..Default::default()
        },
        SkillTemplate {
            id: 31050,
            name: "Moonfall",
            priority: 1,
            min_ratio: 0.1,
            max_ratio: 0.2,
            kind: SkillType::Identity,
            identity_gain: -2.0,
            requires_identity: true,
            cast_duration: Duration::milliseconds(250),
            buffs: vec![
                BuffTemplate {
                    category: BuffCategory::Buff,
                    target: BuffTarget::Party,
                    kind: BuffType::Identity,
                    duration: Duration::seconds(10),
                    value: 0.0
                }
            ],
            cooldown: Duration::seconds(15),
            ..Default::default()
        },
        SkillTemplate {
            id: 31910,
            name: "Masterwork: Efflorescence",
            priority: 13,
            min_ratio: 1.0,
            max_ratio: 2.0,
            identity_gain: 2.0,
            kind: SkillType::Awakening,
            cast_duration: Duration::milliseconds(250),
            buffs: vec![
                BuffTemplate {
                    category: BuffCategory::Buff,
                    target: BuffTarget::Party,
                    kind: BuffType::Shield,
                    duration: Duration::seconds(12),
                    value: 0.0
                },
            ],
            cooldown: Duration::seconds(300),
            cooldown_reduction: 0.515,
            ..Default::default()
        },
        SkillTemplate {
            id: 31930,
            name: "Dream Blossom Garden",
            priority: 2,
            min_ratio: 1.0,
            max_ratio: 2.0,
            identity_gain: 2.0,
            kind: SkillType::HyperAwakening,
            cast_duration: Duration::milliseconds(250),
            buffs: vec![
                BuffTemplate {
                    category: BuffCategory::Buff,
                    target: BuffTarget::Party,
                    kind: BuffType::Other,
                    duration: Duration::seconds(30),
                    value: 0.0
                },
                BuffTemplate {
                    category: BuffCategory::Buff,
                    target: BuffTarget::Party,
                    kind: BuffType::Shield,
                    duration: Duration::seconds(12),
                    value: 0.0
                }
            ],
            ..Default::default()
        }
    ]
}