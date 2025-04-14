mod models;
mod utils;
mod simulator;

use std::{io::{stdout, Write}, thread::sleep, time::{Duration, SystemTime}};
use chrono::Utc;
use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
    ExecutableCommand, QueueableCommand,
};
use anyhow::*;
use models::{player_template::PlayerTemplate, *};
use rand::Rng;
use simulator::Simulator;
use utils::*;

pub fn generate_separator(length: usize) -> String {
    "-".repeat(length) + "\n"
}

fn main() -> Result<()> {
    let mut std_out = stdout();

    let player_templates = vec![
        PlayerTemplate::berserker(),
        PlayerTemplate::deadeye(),
        PlayerTemplate::slayer(),
        PlayerTemplate::bard(),
        PlayerTemplate::sorceress(),
        PlayerTemplate::arcanist(),
        PlayerTemplate::aeromancer(),
        PlayerTemplate::artist(),
    ];

    let mut simulator = Simulator::new(EncounterTemplate::ECHIDNA_G2, player_templates);

    simulator.start();

    let start_time = Utc::now();
    let start_time_formatted = start_time.format("%Y-%m-%d %H:%M:%S").to_string();

    std_out.queue(Clear(ClearType::All))?;

    let separator =  generate_separator(53);
    let mut output = String::with_capacity(1000);

    loop {
        let encounter = simulator.tick();

        let hp_percentage = encounter.boss.hp_percentage * 100.0;
        let formatted_hp = format!("{}/{} ({:.1}%)", format_hp(encounter.boss.current_hp), format_hp(encounter.boss.max_hp), hp_percentage);

    

        std_out.queue(MoveTo(0, 0))?
               .queue(SetForegroundColor(Color::White))?
               .queue(SetBackgroundColor(Color::Black))?;

        output.clear();
        output += separator.as_str();
        output += &format!("| Encounter started: {:<31}|\n", start_time_formatted);
        output += &format!("| Duration: {:<40}|\n", encounter.duration.mmss);
        output += separator.as_str();
        output += &format!("| Boss: {:<44}|\n", encounter.boss.name);
        output += &format!("| HP: {:<46}|\n", formatted_hp);
        output += separator.as_str();
        output += &format!("| {:<19} {:<14} {:<10} {:<15} {:<10} {:<10} {:<10}|\n", "Name", "Class", "Crit", "Damage Dealt", "Brand", "Atk" , "Identity");

        for (i, party) in encounter.parties.iter().enumerate() {
            output += separator.as_str();
            output += &format!("| Party {:<42} |\n", i + 1);
            output += separator.as_str();

            for player in party {
                output += &format!("| {:<19} {:<14} {:<15}|\n", player.name, player.class.as_ref(), format_hp(player.stats.total_damage));
            }
            
        }

        output += separator.as_str();

        std_out.queue(Print(output.clone()))?
               .queue(ResetColor)?
               .flush()?;

        if encounter.boss.current_hp == 0 {
            break;
        }

        sleep(Duration::from_secs(1));
    }

    Ok(())
}
