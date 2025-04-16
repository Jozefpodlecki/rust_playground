use crate::models::class::Class;

use super::{berserker::get_berserker_skills, PlayerTemplate, SkillTemplate};

#[derive(Default, Debug, Clone)]
pub struct PlayerTemplateBuilder {
    name: Option<String>,
    class: Option<Class>,
    crit_rate: f32,
    cooldown_reduction: f32,
    attack_power: u64,
    crit_damage: f32,
    skills: Vec<SkillTemplate>,
}

impl PlayerTemplateBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(mut self, name: &str) -> Self {

        self.name = Some(name.to_string());

        self
    }

    pub fn berserker(mut self) -> Self {

        self.class = Some(Class::Berserker);
        self.skills = get_berserker_skills();
        self.crit_rate = 0.75;
        self.crit_damage = 2.0;
        self.cooldown_reduction = 0.2;
        self.attack_power = 5e6 as u64;

        self
    }
    
    pub fn build(self) -> PlayerTemplate {
        PlayerTemplate {
            name: self.name,
            class: self.class.expect("Provide class"),
            crit_rate: self.crit_rate,
            cooldown_reduction: self.cooldown_reduction,
            attack_power: self.attack_power,
            crit_damage: self.crit_rate,
            skills: self.skills,
        }
    }
}
