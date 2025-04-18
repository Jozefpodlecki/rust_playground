use std::{io::{self, stdout, Read, Write}, panic, thread::sleep, time::{Duration, SystemTime}};
use chrono::Utc;
use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
    ExecutableCommand, QueueableCommand,
};
use anyhow::*;
use crate::{models::{player_template::{PlayerTemplate, PlayerTemplateBuilder}, *}, multi_thread_simulator::MultiThreadSimulator};
use rand::Rng;
use crate::renderer::Renderer;
use crate::utils::*;

pub fn pause() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();

    let _ = stdin.read(&mut [0u8]).unwrap();
}


pub fn run_threaded() -> Result<()> {
    let mut std_out = stdout();

    let player_templates = vec![
        PlayerTemplateBuilder::new().berserker().with_name("Clueless").build(),
        PlayerTemplate::deadeye(),
        PlayerTemplate::slayer(),
        PlayerTemplateBuilder::new().bard().build(),
        PlayerTemplate::reflux_sorceress(),
        PlayerTemplate::arcanist(),
        PlayerTemplate::aeromancer(),
        PlayerTemplateBuilder::new().artist().build(),
        PlayerTemplate::reflux_sorceress(),
        PlayerTemplate::arcanist(),
        PlayerTemplate::aeromancer(),
        PlayerTemplateBuilder::new().paladin().build(),
    ];

    let mut simulator = MultiThreadSimulator::new(EncounterTemplate::ECHIDNA_G2, player_templates);
    let mut renderer = Renderer::new();

    simulator.start();
  
    std_out.queue(Clear(ClearType::All))?;

    loop {
        let encounter = simulator.get_encounter(Duration::from_millis(100));

        let output = renderer.render(&encounter)?;

        std_out.queue(MoveTo(0, 0))?
               .queue(SetForegroundColor(Color::White))?
               .queue(SetBackgroundColor(Color::Black))?;

        std_out.queue(Print(output))?
               .queue(ResetColor)?
               .flush()?;

        if encounter.boss.current_hp == 0 {
            return Ok(());
        }
    }

    pause();
}