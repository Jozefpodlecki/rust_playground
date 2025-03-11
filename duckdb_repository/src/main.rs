use app_config::AppConfig;
use db::repositories::{self, repositories::Repositories};
use orchestrator::Orchestrator;
use simulator::Simulator;
use anyhow::{Result, Ok};

mod db;
mod models;
mod orchestrator;
mod abstractions;
mod simulator;
mod app_config;
mod custom_duration;
mod npc_type;

fn runner() -> Result<()> {
    let app_config = AppConfig::new();
    let repositories = Repositories::new(app_config).unwrap();
    let simulator = Simulator::new(repositories);
    let mut orchestrator = Orchestrator::new(simulator);

    orchestrator.run()?;
 
    Ok(())
}

fn main() {

    match runner() {
        Err(err) => println!("Error: {}", err),
        _ => {},
    }
}
