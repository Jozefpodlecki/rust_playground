use crate::models::class::Class;

use super::{artist::get_artist_skills, bard::get_bard_skills, berserker::get_berserker_skills, paladin::get_paladin_skills, PlayerTemplate, SkillTemplate};

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

    fn apply_support_stats(&mut self) {
        self.crit_rate = 0.1;
        self.crit_damage = 1.0;
        self.cooldown_reduction = 0.45;
        self.attack_power = 5e6 as u64;
    }

    pub fn deadeye(mut self) -> Self {
        self
    }

    pub fn slayer(mut self) -> Self {
        self
    }


    pub fn bard(mut self) -> Self {

        self.apply_support_stats();
        self.class = Some(Class::Bard);
        self.skills = get_bard_skills();

        self
    }

    pub fn reflux_sorceress(mut self) -> Self {
        self
    }

    pub fn aeromancer(mut self) -> Self {
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
        self.skills = get_berserker_skills();
        self.crit_rate = 0.75;
        self.crit_damage = 2.0;
        self.cooldown_reduction = 0.2;
        self.attack_power = 5e6 as u64;

        self
    }

    pub fn gunslinger(mut self) -> Self {

        self.class = Some(Class::Gunslinger);

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
