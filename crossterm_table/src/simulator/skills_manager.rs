use chrono::{DateTime, Duration, Utc};

use crate::models::*;


pub struct SkillsManager {
    skills: Vec<Skill>
}

impl SkillsManager {
    pub fn new(now: DateTime<Utc>, skill_templates: &[SkillTemplate]) -> Self {

        let mut skills = vec![];

        for skill_template in skill_templates {
            let mut ready_on = now;

            if let Some(initial_cooldown) = skill_template.initial_cooldown {
                ready_on += initial_cooldown;
            }

            let skill = Skill {
                id: skill_template.id,
                name: skill_template.name.to_string(),
                ready_on,
                min_ratio: skill_template.min_ratio,
                max_ratio: skill_template.max_ratio,
                identity_gain: skill_template.identity_gain,
                requires_identity: skill_template.requires_identity,
                buffs: skill_template.buffs.clone(),
                priority: skill_template.priority,
                ..Default::default()
            };

            skills.push(skill);
        }

        Self {
            skills,
        }
    }

    pub fn get_available_skills(&self, now: DateTime<Utc>) -> Vec<Skill> {
        self.skills.iter()
            .filter(|s| s.ready_on <= now)
            .cloned()
            .collect::<Vec<_>>()
    }

    pub fn consume(&mut self, skill_id: u32, now: DateTime<Utc>) {
        if let Some(skill) = self.skills.iter_mut().find(|s| s.id == skill_id) {
            skill.ready_on = now + skill.cooldown;
        }
    }

    pub fn get_shortest_cooldown(&self, now: DateTime<Utc>) -> Option<Duration> {
        self.skills
            .iter()
            .filter(|s| s.ready_on > now)
            .map(|s| s.ready_on - now)
            .min()
    }
}