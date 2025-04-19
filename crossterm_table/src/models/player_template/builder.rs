use crate::models::*;

use super::{aeromancer::get_aeromancer_skills, artist::get_artist_skills, bard::get_bard_skills, generic::get_generic_skills, paladin::get_paladin_skills};

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

    pub fn with_min_max_ratio(mut self, min: f32, max: f32) -> Self {

        for skill in self.skills.iter_mut() {
            skill.min_ratio = min;
            skill.max_ratio = max;
        }

        self
    }

    fn apply_support_stats(&mut self) {
        self.crit_rate = 0.1;
        self.crit_damage = 1.0;
        self.cooldown_reduction = 0.45;
        self.attack_power = 5e6 as u64;
    }

    pub fn deadeye(mut self) -> Self {

        self.class = Some(Class::Deadeye);
        self.skills = get_generic_skills();

        self
    }

    pub fn slayer(mut self) -> Self {

        self.class = Some(Class::Sorceress);
        self.skills = get_generic_skills();


        self
    }


    pub fn bard(mut self) -> Self {

        self.apply_support_stats();
        self.class = Some(Class::Bard);
        self.skills = get_bard_skills();

        self
    }

    pub fn sorceress(mut self) -> Self {

        self.class = Some(Class::Sorceress);
        self.skills = get_generic_skills();

        self
    }

    pub fn artist(mut self) -> Self {

        self.apply_support_stats();
        self.class = Some(Class::Bard);
        self.skills = get_artist_skills();

        self
    }

    pub fn paladin(mut self) -> Self {

        self.apply_support_stats();
        self.class = Some(Class::Paladin);
        self.skills = get_paladin_skills();

        self
    }

    pub fn berserker(mut self) -> Self {

        self.class = Some(Class::Berserker);
        self.skills = get_generic_skills();
        self.crit_rate = 0.75;
        self.crit_damage = 2.0;
        self.cooldown_reduction = 0.2;
        self.attack_power = 5e6 as u64;

        self
    }

    pub fn gunslinger(mut self) -> Self {

        self.class = Some(Class::Gunslinger);
        self.skills = get_generic_skills();

        self
    }

    pub fn arcanist(mut self) -> Self {

        self.class = Some(Class::Arcanist);
        self.skills = get_generic_skills();

        self
    }

    pub fn aeromancer(mut self) -> Self {

        self.class = Some(Class::Aeromancer);
        self.skills = get_aeromancer_skills();
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
