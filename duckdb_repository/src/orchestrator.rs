use std::{env, fs};

use chrono::Utc;
use duckdb::DuckdbConnectionManager;
use uuid::Uuid;
use anyhow::{Ok, Result};

use crate::{db::{confrontation_repository::ConfrontationRepository, migration::MigrationRunner, npc_repository::NpcRepository, player_repository::PlayerRepository, raid_repository::RaidRepository}, models::{Npc, Player, Raid, Zone}};

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
    let raid_repository = RaidRepository::new(pool.clone());
    let npc_repository = NpcRepository::new(pool.clone());

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

        player_repository.insert(&player)?;
    }

    let zone = Zone {
        id: 1,
        name: "test".into()
    };

    let raid = Raid {
        id: Uuid::now_v7(),
        name: "".into(),
        sub_name: None,
        created_on: Utc::now(),
        gate: 2,
        zone_ids: vec![]
    };

    let npc = Npc {
        id: Uuid::now_v7(),
        created_on: Utc::now(),
        name: "Test".into(),
        npc_type_id: 1,
        raid_id: Uuid::now_v7(),
    };

    npc_repository.insert(npc);


    Ok(())
}
