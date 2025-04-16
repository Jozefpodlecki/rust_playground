use super::*;

impl PlayerTemplate {
    pub fn artist() -> PlayerTemplate {
        PlayerTemplate {
            class: Class::Artist,
            crit_rate: 0.1,
            crit_damage: 2.0,
            cooldown_reduction: 0.45,
            attack_power: 5e6 as u64,
            skills: vec![
                SkillTemplate {
                    id: 1,
                    name: "Unknown",
                    priority: 3,
                    min_ratio: 0.1,
                    max_ratio: 0.2,
                    identity_gain: 0.1,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 2,
                    name: "Unknown",
                    priority: 4,
                    min_ratio: 0.1,
                    max_ratio: 0.2,
                    identity_gain: 0.1,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 3,
                    name: "Unknown",
                    priority: 5,
                    min_ratio: 0.1,
                    max_ratio: 0.2,
                    identity_gain: 0.1,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 4,
                    name: "Unknown",
                    priority: 6,
                    min_ratio: 0.1,
                    max_ratio: 0.2,
                    identity_gain: 0.1,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 5,
                    name: "Unknown",
                    priority: 7,
                    min_ratio: 0.1,
                    max_ratio: 0.2,
                    identity_gain: 0.1,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 6,
                    name: "Unknown",
                    priority: 8,
                    min_ratio: 0.1,
                    max_ratio: 0.2,
                    identity_gain: 0.1,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 7,
                    name: "Unknown",
                    priority: 9,
                    min_ratio: 0.1,
                    max_ratio: 0.2,
                    identity_gain: 0.1,
                    kind: SkillType::Brand,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: Some(Duration::seconds(10)),
                    buffs: vec![
                        BuffTemplate {
                            category: BuffCategory::Buff,
                            target: BuffTarget::Party,
                            kind: BuffType::Brand,
                            duration: Duration::seconds(10),
                            value: 0.1
                        }
                    ],
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 8,
                    name: "Unknown",
                    priority: 10,
                    min_ratio: 0.1,
                    max_ratio: 0.2,
                    identity_gain: 0.1,
                    kind: SkillType::AttackPowerBuff,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: Some(Duration::seconds(7)),
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
                    id: 9,
                    name: "Unknown",
                    priority: 11,
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
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 10,
                    name: "Unknown",
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
                    id: 11,
                    name: "Unknown",
                    priority: 13,
                    min_ratio: 1.0,
                    max_ratio: 2.0,
                    identity_gain: 2.0,
                    kind: SkillType::Awakening,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(180),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 11,
                    name: "Unknown",
                    priority: 2,
                    min_ratio: 1.0,
                    max_ratio: 2.0,
                    identity_gain: 2.0,
                    kind: SkillType::HyperAwakening,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(180),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    }
}