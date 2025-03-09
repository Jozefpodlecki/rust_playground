use std::{env, fs};

use chrono::Utc;
use duckdb::DuckdbConnectionManager;
use uuid::Uuid;
use anyhow::{Ok, Result};

use crate::{db::{confrontation_repository::ConfrontationRepository, migration::MigrationRunner, player_repository::PlayerRepository}, models::Player};

pub struct Orchestrator {

}

impl Orchestrator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&mut self) -> Result<()> {

        Ok(())
    }
}

fn create_database_and_insert_record() -> Result<()> {
    let version = env!("CARGO_PKG_VERSION");
    let manager = DuckdbConnectionManager::file("db.duckdb")?;
    
    let pool = r2d2::Pool::builder()
        .build(manager)?;
    
    let migration_runner = MigrationRunner::new(pool.clone());
    
    let confrontation_repository = ConfrontationRepository::new(pool.clone());
    let player_repository = PlayerRepository::new(pool.clone());

    migration_runner.run(version)?;
   
    let name = "Alice".to_string();

    if player_repository.exists(&name)? {
        let player = Player {
            id: Uuid::now_v7(),
            name,
            class_id: 101,
            character_id: 1234,
            last_gear_score: 1700.0,
            created_on: Utc::now(),
            updated_on: Utc::now(),
        };

        player_repository.insert(player)?;
    }

    Ok(())
}
