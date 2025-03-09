use duckdb::{params, DuckdbConnectionManager};
use r2d2::Pool;
use anyhow::{Ok, Result};

use crate::models::PlayerStats;

pub struct PlayerStatsRepository {
    pool: Pool<DuckdbConnectionManager>
}

impl PlayerStatsRepository {
    pub fn new(pool: Pool<DuckdbConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn exists(&self, name: &str) -> Result<bool> {

        let connection = self.pool.get()?;

        let sql = r"
        SELECT
            EXISTS (SELECT 1 FROM Player WHERE name = ?)
        ";
    
        let mut statement = connection.prepare(sql)?;
        let params = [name];
        let result = statement.query_row(params, |row| row.get(0))?;
        
        Ok(result)
    }

    pub fn insert(&self, entity: PlayerStats) -> Result<()> {

        let connection = self.pool.get()?;
        let sql = "
        INSERT INTO PlayerStats
        (
            confrontation_id,
            player_id,
            created_on,
            total_damage_taken,
            total_damage_dealt
        )
        VALUES
        (?, ?, ?, ?, ?)";
        let mut statement = connection.prepare(sql).unwrap();

        let params = params![
            entity.confrontation_id.to_string(),
            entity.player_id.to_string(),
            entity.created_on.to_string(),
            entity.total_damage_taken,
            entity.total_damage_dealt
        ];
        statement.execute(params)?;

        Ok(())
    }
}