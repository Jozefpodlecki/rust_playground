use chrono::Duration;

use crate::models::*;


pub fn get_slayer_skills() -> Vec<Skill> {
    let cast_time = Duration::milliseconds(500);

    let skills: [Skill; 10] = [
        Skill {
            id: 45730,
            kind: SkillType::Normal,
            name: "Brutal Impact",
            cast_time,
            cooldown: Duration::seconds(40),
            effects: vec![
                SkillEffect {
                    id: 457301,
                    ratio: 1,
                    kind: SkillEffectType::Damage,
                    buff: None
                },
                SkillEffect {
                    id: 457302,
                    ratio: 1,
                    kind: SkillEffectType::Damage,
                    buff: None
                }
            ]
        },
        Skill {
            id: 45080,
            kind: SkillType::Normal,
            name: "Volcanic Eruption",
            cast_time,
            cooldown: Duration::seconds(36),
            effects: vec![
                SkillEffect {
                    id: 450800,
                    ratio: 1,
                    kind: SkillEffectType::Damage,
                    buff: None
                },
            ]
            
        },
        Skill {
            id: 45720,
            kind: SkillType::Normal,
            name: "Guillotine",
            cast_time,
            cooldown: Duration::seconds(30),
            effects: vec![
                SkillEffect {
                    id: 457200,
                    ratio: 1,
                    kind: SkillEffectType::Damage,
                    buff: None
                },
            ]
        },
        Skill {
            id: 45300,
            kind: SkillType::Normal,
            name: "Fatal Sword",
            cast_time,
            cooldown: Duration::seconds(22),
            effects: vec![
                SkillEffect {
                    id: 453000,
                    ratio: 1,
                    kind: SkillEffectType::Damage,
                    buff: None
                },
            ]
        },
        Skill {
            id: 45060,
            kind: SkillType::Normal,
            name: "Wild Rush",
            cast_time,
            cooldown: Duration::seconds(20),
            effects: vec![
                SkillEffect {
                    id: 450600,
                    ratio: 1,
                    kind: SkillEffectType::Damage,
                    buff: None
                },
            ]
        },
        Skill {
            id: 45710,
            kind: SkillType::Normal,
            name: "Furious Claw",
            cast_time,
            cooldown: Duration::seconds(20),
            effects: vec![
                SkillEffect {
                    id: 457100,
                    ratio: 1,
                    kind: SkillEffectType::Damage,
                    buff: None
                },
                SkillEffect {
                    id: 457110,
                    ratio: 1,
                    kind: SkillEffectType::Damage,
                    buff: None
                },
                SkillEffect {
                    id: 457120,
                    ratio: 1,
                    kind: SkillEffectType::Damage,
                    buff: None
                }
            ]
        },
        Skill {
            id: 45700,
            kind: SkillType::Normal,
            name: "Punishing Draw",
            cast_time,
            cooldown: Duration::seconds(18),
            effects: vec![
                SkillEffect {
                    id: 345700,
                    ratio: 1,
                    kind: SkillEffectType::Damage,
                    buff: None
                }
            ]
        },
        Skill {
            id: 45220,
            kind: SkillType::Normal,
            name: "Wild Stomp",
            cast_time,
            cooldown: Duration::seconds(18),
            effects: vec![
                SkillEffect {
                    id: 457500,
                    ratio: 1,
                    kind: SkillEffectType::Damage,
                    buff: None
                },
                SkillEffect {
                    id: 452203,
                    ratio: 1000,
                    kind: SkillEffectType::AddStatusEffect,
                    buff: Some(SkillBuff {
                        id: 452230,
                        kind: SkillBuffType::DamageAmplification(0.06),
                        duration: Duration::seconds(10),
                    })
                }
                
            ]
        },
        Skill {
            id: 45750,
            kind: SkillType::HyperAwakeningTechnique,
            name: "Spiral Deathblade",
            cast_time,
            cooldown: Duration::seconds(60),
            effects: vec![
                SkillEffect {
                    id: 457500,
                    ratio: 100,
                    kind: SkillEffectType::Damage,
                    buff: None
                }
            ]
        },
        Skill {
            id: 45830,
            kind: SkillType::HyperAwakening,
            name: "Ragna Deathblade",
            cast_time,
            cooldown: Duration::seconds(60),
            effects: vec![
                SkillEffect {
                    id: 458302,
                    ratio: 1000,
                    kind: SkillEffectType::Damage,
                    buff: None
                }
            ]
        },
    ];

    skills.to_vec()
}