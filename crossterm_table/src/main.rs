mod models;
mod utils;
mod simulator;
mod renderer;

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
use models::{player_template::PlayerTemplate, *};
use rand::Rng;
use renderer::Renderer;
use simulator::Simulator;
use utils::*;


fn main() -> Result<()> {
    panic::set_hook(Box::new(|panic_info| {
        eprintln!("Panic occurred: {}", panic_info);
    }));

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
    let mut renderer = Renderer::new();

    simulator.start();
  
    std_out.queue(Clear(ClearType::All))?;

    loop {
        let encounter = simulator.tick();

        let output = renderer.render(encounter);

        std_out.queue(MoveTo(0, 0))?
               .queue(SetForegroundColor(Color::White))?
               .queue(SetBackgroundColor(Color::Black))?;

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
