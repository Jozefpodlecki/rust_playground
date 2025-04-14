use super::*;

impl PlayerTemplate {
    pub fn artist() -> PlayerTemplate {
        PlayerTemplate {
            class: Class::Artist,
            crit_rate: 0.1,
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
                    kind: SkillType::Brand,
                },
                SkillTemplate {
                    id: 8,
                    kind: SkillType::AttackPowerBuff,
                },
                SkillTemplate {
                    id: 9,
                    kind: SkillType::AttackPowerBuff,
                },
                SkillTemplate {
                    id: 10,
                    kind: SkillType::IdentityBuff,
                },
                SkillTemplate {
                    id: 11,
                    kind: SkillType::Awakening,
                },
            ],
        }
    }
}