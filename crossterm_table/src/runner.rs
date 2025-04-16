use std::{io::{stdout, Write}, panic, thread::sleep, time::{Duration, SystemTime}};
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
use crate::simulator::Simulator;
use crate::utils::*;


pub fn run_threaded() -> Result<()> {
    let mut std_out = stdout();

    let player_templates = vec![
        PlayerTemplateBuilder::new().berserker().with_name("Clueless").build(),
        PlayerTemplate::deadeye(),
        PlayerTemplate::slayer(),
        PlayerTemplate::bard(),
        PlayerTemplate::reflux_sorceress(),
        PlayerTemplate::arcanist(),
        PlayerTemplate::aeromancer(),
        PlayerTemplate::artist(),
    ];

    let mut simulator = MultiThreadSimulator::new(EncounterTemplate::ECHIDNA_G2, player_templates);
    let mut renderer = Renderer::new();

    simulator.start();
  
    std_out.queue(Clear(ClearType::All))?;

    loop {
        let encounter = simulator.get_encounter();

        println!("{:#?}", encounter.boss);

        // let output = renderer.render(&encounter);

        // std_out.queue(MoveTo(0, 0))?
        //        .queue(SetForegroundColor(Color::White))?
        //        .queue(SetBackgroundColor(Color::Black))?;

        // std_out.queue(Print(output.clone()))?
        //        .queue(ResetColor)?
        //        .flush()?;

        // if encounter.boss.current_hp == 0 {
        //     return Ok(());
        // }

        sleep(Duration::from_secs(1));
    }
}

pub fn run() -> Result<()> {
    let mut std_out = stdout();

    let player_templates = vec![
        PlayerTemplateBuilder::new().berserker().with_name("Clueless").build(),
        PlayerTemplate::deadeye(),
        PlayerTemplate::slayer(),
        PlayerTemplate::bard(),
        PlayerTemplate::reflux_sorceress(),
        PlayerTemplate::arcanist(),
        PlayerTemplate::aeromancer(),
        PlayerTemplate::artist(),
    ];

    let mut simulator = Simulator::new(EncounterTemplate::ECHIDNA_G2, player_templates);
    let mut renderer = Renderer::new();

    simulator.start();
  
    std_out.queue(Clear(ClearType::All))?;

    loop {
        simulator.progress();
        let encounter = simulator.get_encounter();

        let output = renderer.render(encounter);

        std_out.queue(MoveTo(0, 0))?
               .queue(SetForegroundColor(Color::White))?
               .queue(SetBackgroundColor(Color::Black))?;

        std_out.queue(Print(output.clone()))?
               .queue(ResetColor)?
               .flush()?;

        if encounter.boss.current_hp == 0 {
            return Ok(());
        }

        sleep(Duration::from_secs(1));
    }
}