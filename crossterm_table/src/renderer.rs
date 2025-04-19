use crate::{models::encounter::Encounter, utils::{format_unit, generate_separator}};
use std::fmt::Write;
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

        self.render_header(encounter)?;
        self.render_boss(encounter)?;
        self.render_global_stats(encounter)?;
        self.render_parties(encounter)?;

        Ok(&self.buffer)
    }

    fn render_header(&mut self, encounter: &Encounter) -> Result<()> {
        let sep = generate_separator(86);
        let start_time = encounter.started_on.format("%Y-%m-%d %H:%M:%S");

        write!(self.buffer, "{}| Encounter started: {:<64}|\n", sep, start_time)?;
        write!(self.buffer, "| Duration: {:<73}|\n", encounter.duration.mmss)?;
        write!(self.buffer, "{}", sep)?;
        Ok(())
    }

    fn render_boss(&mut self, encounter: &Encounter) -> Result<()> {
        let boss = &encounter.boss;
        let hp_pct = boss.hp_percentage * 100.0;
        let hp_bars = format!("{:.1}/{}", boss.hp_bars, boss.max_hp_bars);
        let hp_line = format!(
            "{}/{} {} ({:.1}%)",
            format_unit(boss.current_hp),
            format_unit(boss.max_hp),
            hp_bars,
            hp_pct
        );

        write!(self.buffer, "| Boss: {:<77}|\n", boss.name)?;
        write!(self.buffer, "| HP: {:<79}|\n", hp_line)?;
        write!(self.buffer, "{}", generate_separator(86))?;
        Ok(())
    }

    fn render_global_stats(&mut self, encounter: &Encounter) -> Result<()> {
        let stats = &encounter.stats;
        write!(self.buffer, "| DPS: {:<78}|\n", format_unit(stats.dps))?;
        write!(self.buffer, "| TTK: {:<78}|\n", stats.ttk)?;
        write!(self.buffer, "{}", generate_separator(86))?;
        write!(
            self.buffer,
            "| {:<19}{:<14}{:<8}{:<8}{:<9}{:<8}{:<8}{:<8} |\n",
            "Name", "Class", "%", "Crit", "DPS", "Brand", "Atk", "Identity"
        )?;
        Ok(())
    }

    fn render_parties(&mut self, encounter: &Encounter) -> Result<()> {
        for (i, party) in encounter.parties.iter().enumerate() {
            let sep = generate_separator(86);
            write!(self.buffer, "{}", sep)?;
            write!(
                self.buffer,
                "| Party {} {:<7} {:<8} {:<57} |\n",
                i + 1,
                format_unit(party.stats.dps),
                format!("({:.1}%)", party.stats.total_damage_percentage * 100.0),
                ""
            )?;
            write!(self.buffer, "{}", sep)?;

            for player in &party.players {
                self.render_player_line(player)?;
            }
        }

        write!(self.buffer, "{}", generate_separator(86))?;
        Ok(())
    }

    fn render_player_line(&mut self, player: &crate::models::Player) -> Result<()> {
        let stats = &player.stats;
        write!(
            self.buffer,
            "| {:<19}{:<14}{:<8}{:<8}{:<9}{:<8}{:<8}{:<8} |\n",
            player.name,
            player.class.as_ref(),
            format!("{:.1}%", stats.total_damage_percentage * 100.0),
            format!("{:.1}%", stats.crit_rate * 100.0),
            format_unit(stats.dps),
            format!("{:.1}%", stats.brand_percentage * 100.0),
            format!("{:.1}%", stats.attack_power_buff_percentage * 100.0),
            format!("{:.1}%", stats.identity_percentage * 100.0),
        )?;
        Ok(())
    }
}
