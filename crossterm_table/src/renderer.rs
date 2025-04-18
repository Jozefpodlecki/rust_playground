use crate::{models::Encounter, utils::{format_unit, generate_separator}};
use std::fmt::Write; // For `write!` macro
use anyhow::*;

pub struct Renderer {
    buffer: String,
}

impl Renderer {
    pub fn new() -> Self {
        let buffer = String::with_capacity(1000);
        Self { buffer }
    }

    pub fn render(&mut self, encounter: &Encounter) -> Result<&String> {
        self.buffer.clear();

        let hp_percentage = encounter.boss.hp_percentage * 100.0;
        let start_time_formatted = encounter.started_on.format("%Y-%m-%d %H:%M:%S").to_string();

        let formatted_hp_and_bars = format!(
            "{}/{} {:.1}/{} ({:.1}%)",
            format_unit(encounter.boss.current_hp),
            format_unit(encounter.boss.max_hp),
            encounter.boss.hp_bars,
            encounter.boss.max_hp_bars,
            hp_percentage
        );

        let separator = generate_separator(86);

        write!(self.buffer, "{}", separator)?;
        write!(self.buffer, "| Encounter started: {:<64}|\n", start_time_formatted)?;
        write!(self.buffer, "| Duration: {:<73}|\n", encounter.duration.mmss)?;
        write!(self.buffer, "{}", separator)?;
        write!(self.buffer, "| Boss: {:<77}|\n", encounter.boss.name)?;
        write!(self.buffer, "| HP: {:<79}|\n", formatted_hp_and_bars)?;
        write!(self.buffer, "{}", separator)?;
        write!(self.buffer, "| DPS: {:<78}|\n", format_unit(encounter.stats.dps))?;
        write!(self.buffer, "| TTK: {:<78}|\n", encounter.stats.ttk)?;
        write!(self.buffer, "{}", separator)?;
        write!(
            self.buffer,
            "| {:<19}{:<14}{:<8}{:<8}{:<9}{:<8}{:<8}{:<8} |\n",
            "Name", "Class", "%", "Crit", "DPS", "Brand", "Atk", "Identity"
        )?;

        for (i, party) in encounter.parties.iter().enumerate() {
            write!(self.buffer, "{}", separator)?;
            write!(
                self.buffer,
                "| Party {} {:<7} {:<8} {:<57} |\n",
                i + 1,
                format_unit(party.stats.dps),
                format!("({:.1}%)", party.stats.total_damage_percentage * 100.0),
                ""
            )?;
            write!(self.buffer, "{}", separator)?;

            for player in &party.players {
                write!(
                    self.buffer,
                    "| {:<19}{:<14}{:<8}{:<8}{:<9}{:<8}{:<8}{:<8} |\n",
                    player.name,
                    player.class.as_ref(),
                    format!("{:.1}%", player.stats.total_damage_percentage * 100.0),
                    format!("{:.1}%", player.stats.crit_rate * 100.0),
                    format_unit(player.stats.dps),
                    format!("{:.1}%", player.stats.brand_percentage * 100.0),
                    format!("{:.1}%", player.stats.attack_power_buff_percentage * 100.0),
                    format!("{:.1}%", player.stats.identity_percentage * 100.0),
                )?;
            }
        }

        write!(self.buffer, "{}", separator)?;

        Ok(&self.buffer)
    }
}
