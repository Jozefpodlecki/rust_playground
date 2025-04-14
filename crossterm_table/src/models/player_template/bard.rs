use super::*;

impl PlayerTemplate {
    pub fn bard() -> PlayerTemplate {
        PlayerTemplate {
            class: Class::Bard,
            crit_rate: 0.1,
            cooldown_reduction: 0.45,
            attack_power: 5e6 as u64,
            skills: vec![
                SkillTemplate {
                    id: 21290,
                    name: "Sonatina",
                    priority: 2,
                    kind: SkillType::Brand,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: Some(Duration::seconds(3)),
                    cooldown: Duration::seconds(21),
                },
                SkillTemplate {
                    id: 2,
                    name: "Unknown",
                    priority: 2,
                    kind: SkillType::AttackPowerBuff,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: Some(Duration::seconds(7)),
                    cooldown: Duration::seconds(15),
                },
                SkillTemplate {
                    id: 3,
                    name: "Unknown",
                    priority: 2,
                    kind: SkillType::AttackPowerBuff,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: Some(Duration::seconds(7)),
                    cooldown: Duration::seconds(15),
                },
                SkillTemplate {
                    id: 4,
                    name: "Unknown",
                    priority: 2,
                    kind: SkillType::Identity,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: Some(Duration::seconds(10)),
                    cooldown: Duration::seconds(15),
                },
                SkillTemplate {
                    id: 5,
                    name: "Prelude of Storm",
                    priority: 2,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(15),
                },
                SkillTemplate {
                    id: 6,
                    name: "Unknown",
                    priority: 2,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(15),
                },
                SkillTemplate {
                    id: 21180,
                    name: "Harp of Rhythm",
                    priority: 2,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(15),
                },
                SkillTemplate {
                    id: 8,
                    name: "Unknown",
                    priority: 2,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(15),
                },
                SkillTemplate {
                    id: 9,
                    name: "Unknown",
                    priority: 2,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(15),
                },
                SkillTemplate {
                    id: 10,
                    name: "Unknown",
                    priority: 2,
                    kind: SkillType::HyperAwakeningTechnique,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: Some(Duration::seconds(20)),
                    cooldown: Duration::seconds(40),
                },
                SkillTemplate {
                    id: 11,
                    name: "Unknown",
                    priority: 1,
                    kind: SkillType::Awakening,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(15),
                },
            ],
        }
    }
}