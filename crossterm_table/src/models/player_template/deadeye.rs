use super::*;

impl PlayerTemplate {
    pub fn deadeye() -> PlayerTemplate {
        PlayerTemplate {
            class: Class::Deadeye,
            crit_rate: 0.75,
            crit_damage: 2.0,
            cooldown_reduction: 0.4,
            attack_power: 5e6 as u64,
            skills: vec![
                SkillTemplate {
                    id: 1,
                    name: "Unknown",
                    priority: 2,
                    min_ratio: 10.0,
                    max_ratio: 20.0,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 2,
                    name: "Unknown",
                    priority: 3,
                    min_ratio: 10.0,
                    max_ratio: 20.0,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 3,
                    name: "Unknown",
                    priority: 4,
                    min_ratio: 10.0,
                    max_ratio: 20.0,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 4,
                    name: "Unknown",
                    priority: 5,
                    min_ratio: 10.0,
                    max_ratio: 20.0,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 5,
                    name: "Unknown",
                    priority: 6,
                    min_ratio: 10.0,
                    max_ratio: 20.0,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 6,
                    name: "Unknown",
                    priority: 7,
                    min_ratio: 10.0,
                    max_ratio: 20.0,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 7,
                    name: "Unknown",
                    priority: 8,
                    min_ratio: 10.0,
                    max_ratio: 20.0,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 8,
                    name: "Unknown",
                    priority: 9,
                    min_ratio: 10.0,
                    max_ratio: 20.0,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 9,
                    name: "Unknown",
                    priority: 10,
                    min_ratio: 10.0,
                    max_ratio: 20.0,
                    kind: SkillType::Normal,
                    cast_duration: Duration::milliseconds(250),
                    buff_duration: None,
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
                SkillTemplate {
                    id: 10,
                    name: "Unknown",
                    priority: 1,
                    min_ratio: 100.0,
                    max_ratio: 200.0,
                    kind: SkillType::HyperAwakening,
                    cast_duration: Duration::milliseconds(2),
                    buff_duration: None,
                    cooldown: Duration::seconds(15),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    }
}