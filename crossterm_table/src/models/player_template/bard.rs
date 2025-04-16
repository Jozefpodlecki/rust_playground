use super::*;

impl PlayerTemplate {
    pub fn bard() -> PlayerTemplate {
        PlayerTemplate {
            class: Class::Bard,
            crit_rate: 0.1,
            crit_damage: 1.0,
            cooldown_reduction: 0.45,
            attack_power: 5e6 as u64,
            skills: vec![
                SkillTemplate {
                    id: 21290,
                    name: "Sonatina",
                    priority: 2,
                    kind: SkillType::Brand,
                    identity_gain: 0.2,
                    min_ratio: 1.0,
                    max_ratio: 2.0,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: Some(Duration::seconds(3)),
                    buffs: vec![
                        BuffTemplate {
                            category: BuffCategory::Debuff,
                            target: BuffTarget::Party,
                            kind: BuffType::Brand,
                            duration: Duration::seconds(3),
                            value: 0.0
                        }
                    ],
                    cooldown: Duration::seconds(21),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 21170,
                    name: "Sonic Vibration",
                    priority: 3,
                    identity_gain: 0.1,
                    min_ratio: 1.0,
                    max_ratio: 2.0,
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
                    cooldown: Duration::seconds(24),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 21140,
                    name: "Heavenly Tune",
                    priority: 4,
                    identity_gain: 0.1,
                    min_ratio: 1.0,
                    max_ratio: 2.0,
                    kind: SkillType::AttackPowerBuff,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: Some(Duration::seconds(7)),
                    buffs: vec![
                        BuffTemplate {
                            category: BuffCategory::Buff,
                            target: BuffTarget::Party,
                            kind: BuffType::Identity,
                            duration: Duration::seconds(7),
                            value: 0.0
                        }
                    ],
                    cooldown: Duration::seconds(30),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 21140,
                    name: "Serenade of Courage",
                    priority: 5,
                    kind: SkillType::Identity,
                    identity_gain: -1.0,
                    min_ratio: 1.0,
                    max_ratio: 2.0,
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
                    id: 21080,
                    name: "Prelude of Storm",
                    priority: 6,
                    identity_gain: 0.3,
                    min_ratio: 1.0,
                    max_ratio: 2.0,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(16),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 6,
                    name: "Guardian Tune",
                    priority: 7,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 21180,
                    name: "Harp of Rhythm",
                    priority: 8,
                    identity_gain: 0.1,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 21070,
                    name: "Wind of Music",
                    priority: 9,
                    identity_gain: 0.3,
                    min_ratio: 1.0,
                    max_ratio: 2.0,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 9,
                    name: "Guardian Tune",
                    priority: 10,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 21300,
                    name: "Aria",
                    priority: 11,
                    identity_gain: 0.1,
                    min_ratio: 1.0,
                    max_ratio: 2.0,
                    kind: SkillType::HyperAwakeningTechnique,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: Some(Duration::seconds(20)),
                    cooldown: Duration::seconds(90),
                    buffs: vec![
                        BuffTemplate {
                            category: BuffCategory::Buff,
                            target: BuffTarget::Party,
                            kind: BuffType::HyperAwakeningTechnique,
                            duration: Duration::seconds(20),
                            value: 0.0
                        }
                    ],
                    ..Default::default()
                },
                SkillTemplate {
                    id: 21320,
                    name: "Symphonia",
                    priority: 1,
                    identity_gain: 1.0,
                    min_ratio: 1.0,
                    max_ratio: 2.0,
                    kind: SkillType::Awakening,
                    cast_duration: Duration::seconds(1),
                    buff_duration: None,
                    cooldown: Duration::seconds(300),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 21320,
                    name: "Symphony Melody",
                    priority: 1,
                    identity_gain: 1.0,
                    min_ratio: 1.0,
                    max_ratio: 2.0,
                    kind: SkillType::Awakening,
                    cast_duration: Duration::seconds(3),
                    buff_duration: None,
                    cooldown: Duration::seconds(300),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    }
}