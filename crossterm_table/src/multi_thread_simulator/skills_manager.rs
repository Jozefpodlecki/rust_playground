use crate::models::{player_template::SkillTemplate, Skill};


pub struct SkillsManager {
    skills: Vec<Skill>
}

impl SkillsManager {
    pub fn new(skill_templates: &[SkillTemplate]) -> Self {

        let mut skills = vec![];

        for skill_template in skill_templates {
            let skill = Skill {
                id: skill_template.id,
                name: skill_template.name.to_string()
            };

            skills.push(skill);
        }

        Self {
            skills,
        }
    }
}