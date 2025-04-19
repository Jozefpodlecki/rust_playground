use crate::models::*;
use super::*;

pub fn get_artillerist_skills() -> Vec<SkillTemplate> {
    vec![
        SkillTemplate {
            id: 1,
            name: "Unknown",
            priority: 2,
            kind: SkillType::Normal,
            cast_duration: Duration::milliseconds(250),
            cooldown: Duration::seconds(15),
            ..Default::default()
        },
        SkillTemplate {
            id: 2,
            name: "Unknown",
            priority: 2,
            kind: SkillType::Normal,
            cast_duration: Duration::milliseconds(250),
            cooldown: Duration::seconds(15),
            ..Default::default()
        },
        SkillTemplate {
            id: 3,
            name: "Unknown",
            priority: 2,
            kind: SkillType::Normal,
            cast_duration: Duration::milliseconds(250),
            cooldown: Duration::seconds(15),
            ..Default::default()
        },
        SkillTemplate {
            id: 4,
            name: "Unknown",
            priority: 2,
            kind: SkillType::Normal,
            cast_duration: Duration::milliseconds(250),
            cooldown: Duration::seconds(15),
            ..Default::default()
        },
        SkillTemplate {
            id: 5,
            name: "Unknown",
            priority: 2,
            kind: SkillType::Normal,
            cast_duration: Duration::milliseconds(250),
            cooldown: Duration::seconds(15),
            ..Default::default()
        },
        SkillTemplate {
            id: 6,
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