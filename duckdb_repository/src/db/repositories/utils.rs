use chrono::Utc;
use duckdb::DuckdbConnectionManager;
use r2d2::Pool;
use anyhow::{Ok, Result};
use rand::{rng, Rng};
use uuid::Uuid;

use crate::{abstractions::FileSystem, db::migration::MigrationRunner, models::*, npc_type::NpcType};

use super::*;

pub fn setup_test_database() -> Result<Pool<DuckdbConnectionManager>> {
    let pool = setup_database("0.1.0")?;

    Ok(pool)
}

pub fn setup_database(version: &str) -> Result<Pool<DuckdbConnectionManager>> {
    let manager = DuckdbConnectionManager::memory()?;
    let pool = Pool::new(manager)?;
    let config_repository = ConfigRepository::new(pool.clone());
    let file_system = FileSystem::new();
    let migration_runner = MigrationRunner::new(pool.clone(), config_repository, file_system);
    migration_runner.run(version);

    Ok(pool)
}

fn random_u8_range(start: u8, end: u8) -> u8 {
    let mut rng = rng();
    rng.random_range(start..=end)
}

fn random_u32_range(start: u32, end: u32) -> u32 {
    let mut rng = rng();
    rng.random_range(start..=end)
}

fn random_u64_range(start: u64, end: u64) -> u64 {
    let mut rng = rng();
    rng.random_range(start..=end)
}

fn random_i64_range(start: i64, end: i64) -> i64 {
    let mut rng = rng();
    rng.random_range(start..=end)
}

fn generate_random_string(len: usize) -> String {
    let mut rng = rng();
    let first_letter = rng.random_range(b'A'..=b'Z') as char;
    
    let rest: String = (0..len-1)
        .map(|_| {
            let c = rng.random_range(0..52);
            if c < 26 {
                (b'a' + c as u8) as char
            } else {
                (b'A' + (c - 26) as u8) as char
            }
        })
        .collect();

    format!("{}{}", first_letter, rest)
}

pub struct TestDb {
    pub pool: Pool<DuckdbConnectionManager>,
    pub config_repository: Option<ConfigRepository>,
    pub npc_repository: Option<NpcRepository>,
    pub player_repository: Option<PlayerRepository>,
    pub player_stats_repository: Option<PlayerStatsRepository>,
    pub raid_repository: Option<RaidRepository>,
    pub confrontation_repository: Option<ConfrontationRepository>,
    pub hp_session_repository: Option<HpSessionRepository>,
}

impl TestDb {
    pub fn new() -> Self {
        let manager = DuckdbConnectionManager::memory().unwrap();
        let pool = Pool::new(manager).unwrap();

        Self {
            pool,
            config_repository: None,
            npc_repository: None,
            player_repository: None,
            player_stats_repository: None,
            raid_repository: None,
            confrontation_repository: None,
            hp_session_repository: None
        }
    }

    pub fn setup(&mut self) -> Result<()> {
        
        let config_repository = ConfigRepository::new(self.pool.clone());
        let file_system = FileSystem::new();
        let migration_runner = MigrationRunner::new(self.pool.clone(), config_repository, file_system);
        migration_runner.run("0.1.0")?;

        let config_repository = ConfigRepository::new(self.pool.clone());
        self.config_repository.get_or_insert(config_repository);

        Ok(())
    }

    pub fn create_player(&mut self) -> Result<Player> {

        let player = Player {
            id: Uuid::now_v7(),
            updated_on: Utc::now(),
            created_on: Utc::now(),
            character_id: random_u64_range(1000, 10000),
            class_id: 1,
            last_gear_score: 1670.0,
            name: generate_random_string(10),
        };

        let player_repository = self.player_repository.get_or_insert_with(|| PlayerRepository::new(self.pool.clone()));
        player_repository.insert(&player)?;

        Ok(player)
    }

    pub fn create_npc(&mut self, raid_id: Uuid) -> Result<Npc> {
        let npc = Npc {
            id: Uuid::now_v7(),
            created_on: Utc::now(),
            name: generate_random_string(10),
            npc_id: random_u32_range(1000, 10000),
            npc_type: NpcType::Boss,
            raid_id,
        };

        let npc_repository = self.npc_repository.get_or_insert_with(|| NpcRepository::new(self.pool.clone()));
        npc_repository.insert(&npc)?;
        
        Ok(npc)
    }

    pub fn create_raid(&mut self) -> Result<Raid> {
        let raid = Raid {
            id: Uuid::now_v7(),
            created_on: Utc::now(),
            name: generate_random_string(10),
            sub_name: None,
            gate: random_u8_range(1, 4),
            zone_ids: vec![random_u32_range(1000, 10000)],
        };

        let raid_repository = self.raid_repository.get_or_insert_with(|| RaidRepository::new(self.pool.clone()));
        raid_repository.insert(&raid)?;
        
        Ok(raid)
    }

    pub fn create_confrontation(&mut self, raid_id: Uuid) -> Result<Confrontation> {
        
        let confrontation = Confrontation {
            id: Uuid::now_v7(),
            is_cleared: false,
            created_on: Utc::now(),
            raid_id,
            total_damage_taken: 100,
            total_damage_dealt: 100,
            duration: random_u64_range(100, 1000).into()
        };

        let confrontation_repository = self.confrontation_repository.get_or_insert_with(|| ConfrontationRepository::new(self.pool.clone()));
        confrontation_repository.insert(&confrontation)?;

        Ok(confrontation)
    }
    
    pub fn create_hp_session(&mut self, confrontation_id: Uuid, npc_id: Uuid) -> Result<HpSession> {
     
        let hp_session = HpSession {
            id: Uuid::now_v7(),
            confrontation_id,
            npc_id,
            started_on: Utc::now(),
            ended_on: None
        };

        let hp_session_repository = self.hp_session_repository.get_or_insert_with(|| HpSessionRepository::new(self.pool.clone()));
        hp_session_repository.insert(&hp_session)?;

        Ok(hp_session)
    }
}