use crate::{models::Encounter, utils::{format_unit, generate_separator}};

pub struct Renderer {
    buffer: String
}

impl Renderer {
    pub fn new() -> Self {
        let buffer = String::with_capacity(1000);

        Self { buffer }
    }

    pub fn render(&mut self, encounter: &Encounter) -> &String {
        self.buffer.clear();

        let hp_percentage = encounter.boss.hp_percentage * 100.0;
        let start_time_formatted = encounter.started_on.format("%Y-%m-%d %H:%M:%S").to_string();
        let formatted_hp = format!("{}/{} ({:.1}%)", format_unit(encounter.boss.current_hp), format_unit(encounter.boss.max_hp), hp_percentage);

        let separator =  generate_separator(78);

        self.buffer += separator.as_str();
        self.buffer += &format!("| Encounter started: {:<56}|\n", start_time_formatted);
        self.buffer += &format!("| Duration: {:<65}|\n", encounter.duration.mmss);
        self.buffer += separator.as_str();
        self.buffer += &format!("| Boss: {:<69}|\n", encounter.boss.name);
        self.buffer += &format!("| HP: {:<71}|\n", formatted_hp);
        self.buffer += separator.as_str();
        self.buffer += &format!("| DPS: {:<70}|\n", format_unit(encounter.stats.dps));
        self.buffer += &format!("| TTK: {:<70}|\n", encounter.stats.ttk);
        self.buffer += separator.as_str();
        self.buffer += &format!("| {:<19}{:<14}{:<8}{:<9}{:<8}{:<8}{:<8} |\n", "Name", "Class", "Crit", "DPS", "Brand", "Atk" , "Identity");

        for (i, party) in encounter.parties.iter().enumerate() {
            self.buffer += separator.as_str();
            self.buffer += &format!("| Party {} {:<66} |\n", i + 1, format_unit(party.stats.dps));
            self.buffer += separator.as_str();

            for player in party.players.iter() {
                self.buffer += &format!("| {:<19}{:<14}{:<8}{:<9}{:<8}{:<8}{:<8} |\n",
                    player.name,
                    player.class.as_ref(),
                    format!("{:.1}%", player.stats.crit_rate * 100.0),
                    format_unit(player.stats.dps),
                    format!("{:.1}%", player.stats.brand_percentage * 100.0),
                    format!("{:.1}%", player.stats.attack_power_buff_percentage * 100.0),
                    format!("{:.1}%", player.stats.identity_percentage * 100.0),
                );
            }
            
        }

        self.buffer += separator.as_str();
        
        &self.buffer
    }
}