use super::*;

    pub fn get_paladin_skills() -> Vec<SkillTemplate> {
        vec![
            SkillTemplate {
                id: 36050,
                name: "Light Shock",
                priority: 2,
                identity_gain: 0.1,
                kind: SkillType::Normal,
                cast_duration: Duration::milliseconds(250),
                cooldown: Duration::seconds(8),
                ..Default::default()
            },
            SkillTemplate {
                id: 36080,
                name: "Sword of Justice",
                priority: 3,
                identity_gain: 0.1,
                kind: SkillType::Normal,
                cast_duration: Duration::milliseconds(250),
                buffs: vec![
                    BuffTemplate {
                        category: BuffCategory::Debuff,
                        target: BuffTarget::Party,
                        kind: BuffType::Brand,
                        duration: Duration::seconds(10),
                        value: 0.0
                    }
                ],
                cooldown: Duration::seconds(15),
                ..Default::default()
            },
            SkillTemplate {
                id: 36060,
                name: "Light of Judgment",
                priority: 4,
                identity_gain: 0.2,
                kind: SkillType::Normal,
                cast_duration: Duration::milliseconds(250),
                cooldown: Duration::seconds(15),
                ..Default::default()
            },
            SkillTemplate {
                id: 36150,
                name: "God’s Decree",
                priority: 5,
                kind: SkillType::Normal,
                cast_duration: Duration::milliseconds(250),
                cooldown: Duration::seconds(20),
                ..Default::default()
            },
            SkillTemplate {
                id: 36170,
                name:  "Wrath of God",
                priority: 6,
                kind: SkillType::Normal,
                cast_duration: Duration::milliseconds(250),
                cooldown: Duration::seconds(27),
                cooldown_gem: 0.18,
                ..Default::default()
            },
            SkillTemplate {
                id: 36140,
                name: "Unknown",
                priority: 2,
                kind: SkillType::Normal,
                cast_duration: Duration::milliseconds(250),
                cooldown: Duration::seconds(15),
                ..Default::default()
            },
            SkillTemplate {
                id: 7,
                name: "Unknown",
                priority: 2,
                kind: SkillType::Normal,
                cast_duration: Duration::milliseconds(250),
                cooldown: Duration::seconds(15),
                ..Default::default()
            },
            SkillTemplate {
                id: 8,
                name: "Unknown",
                priority: 2,
                kind: SkillType::Normal,
                cast_duration: Duration::milliseconds(250),
                cooldown: Duration::seconds(15),
                ..Default::default()
            },
            SkillTemplate {
                id: 9,
                name: "Unknown",
                priority: 2,
                kind: SkillType::Normal,
                cast_duration: Duration::milliseconds(250),
                cooldown: Duration::seconds(15),
                ..Default::default()
            },
            SkillTemplate {
                id: 10,
                name: "Unknown",
                priority: 2,
                kind: SkillType::Awakening,
                cast_duration: Duration::milliseconds(250),
                cooldown: Duration::seconds(15),
                ..Default::default()
            },
        ]
}