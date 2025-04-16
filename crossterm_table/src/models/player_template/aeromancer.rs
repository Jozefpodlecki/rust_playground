use super::*;

impl PlayerTemplate {
    pub fn aeromancer() -> PlayerTemplate {
        PlayerTemplate {
            class: Class::Aeromancer,
            crit_rate: 0.75,
            crit_damage: 2.0,
            cooldown_reduction: 0.4,
            attack_power: 5e6 as u64,
            skills: vec![
                SkillTemplate {
                    id: 1,
                    name: "Unknown",
                    priority: 2,
                    min_ratio: 1.0,
                    max_ratio: 2.0,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 2,
                    name: "Unknown",
                    priority: 2,
                    min_ratio: 1.0,
                    max_ratio: 2.0,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 3,
                    name: "Unknown",
                    priority: 2,
                    min_ratio: 1.0,
                    max_ratio: 2.0,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(30),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 4,
                    name: "Unknown",
                    priority: 2,
                    min_ratio: 1.0,
                    max_ratio: 2.0,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(30),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 5,
                    name: "Unknown",
                    priority: 2,
                    min_ratio: 1.0,
                    max_ratio: 2.0,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(30),
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
                    buff_duration: None,
                    cooldown: Duration::seconds(30),
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
                    buff_duration: None,
                    cooldown: Duration::seconds(30),
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
                    buff_duration: None,
                    cooldown: Duration::seconds(30),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 9,
                    name: "Unknown",
                    priority: 2,
                    min_ratio: 1.0,
                    max_ratio: 2.0,
                    kind: SkillType::HyperAwakeningTechnique,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(30),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 10,
                    name: "Unknown",
                    priority: 1,
                    min_ratio: 1000.0,
                    max_ratio: 2000.0,
                    kind: SkillType::HyperAwakening,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(30),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    }
}