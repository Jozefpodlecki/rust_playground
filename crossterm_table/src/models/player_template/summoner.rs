use super::*;

impl PlayerTemplate {
    pub fn summoner() -> PlayerTemplate {
        PlayerTemplate {
            class: Class::Berserker,
            crit_rate: 0.75,
            cooldown_reduction: 0.4,
            attack_power: 5e6 as u64,
            skills: vec![
                SkillTemplate {
                    id: 1,
                    kind: SkillType::Normal,
                },
                SkillTemplate {
                    id: 2,
                    kind: SkillType::Normal,
                },
                SkillTemplate {
                    id: 3,
                    kind: SkillType::Normal,
                },
                SkillTemplate {
                    id: 4,
                    kind: SkillType::Normal,
                },
                SkillTemplate {
                    id: 5,
                    kind: SkillType::Normal,
                },
                SkillTemplate {
                    id: 6,
                    kind: SkillType::Normal,
                },
                SkillTemplate {
                    id: 7,
                    kind: SkillType::Normal,
                },
                SkillTemplate {
                    id: 8,
                    kind: SkillType::Normal,
                },
                SkillTemplate {
                    id: 9,
                    kind: SkillType::Normal,
                },
                SkillTemplate {
                    id: 10,
                    kind: SkillType::Awakening,
                },
            ],
        }
    }
}