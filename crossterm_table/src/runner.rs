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
use crate::{models::{player_template::{PlayerTemplateBuilder}, *}, simulator::Simulator};
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
        PlayerTemplateBuilder::new().deadeye().build(),
        PlayerTemplateBuilder::new().slayer().build(),
        PlayerTemplateBuilder::new().bard().build(),
        PlayerTemplateBuilder::new().sorceress().build(),
        PlayerTemplateBuilder::new().arcanist().build(),
        PlayerTemplateBuilder::new().aeromancer().build(),
        PlayerTemplateBuilder::new().artist().build(),
        PlayerTemplateBuilder::new().sorceress().build(),
        PlayerTemplateBuilder::new().arcanist().build(),
        PlayerTemplateBuilder::new().aeromancer().build(),
        PlayerTemplateBuilder::new().paladin().build(),
        PlayerTemplateBuilder::new().sorceress().build(),
        PlayerTemplateBuilder::new().arcanist().build(),
        PlayerTemplateBuilder::new().aeromancer().build(),
        PlayerTemplateBuilder::new().paladin().build(),
    ];

    let mut simulator = Simulator::new(EncounterTemplate::BEHEMOTH_G1, player_templates);
    let mut renderer = Renderer::new();

    simulator.start();
  
    std_out.queue(Clear(ClearType::All))?;

    loop {
        // let encounter = simulator.get_encounter(Duration::from_millis(100));

        // let output = renderer.render(&encounter)?;

        // std_out.queue(MoveTo(0, 0))?
        //        .queue(SetForegroundColor(Color::White))?
        //        .queue(SetBackgroundColor(Color::Black))?;

        // std_out.queue(Print(output))?
        //        .queue(ResetColor)?
        //        .flush()?;

        // if simulator.has_ended() {
        //     return Ok(());
        // }
    }

    pause();
}