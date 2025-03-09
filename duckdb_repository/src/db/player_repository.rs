use duckdb::{params, DuckdbConnectionManager};
use r2d2::Pool;
use anyhow::{Ok, Result};

use crate::models::Player;

pub struct PlayerRepository {
    pool: Pool<DuckdbConnectionManager>
}

impl PlayerRepository {
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

    pub fn insert(&self, entity: Player) -> Result<()> {

        let connection = self.pool.get()?;
        let sql = "
        INSERT INTO Player
        (
            id,
            name,
            class_id,
            character_id,
            last_gear_score,
            created_on,
            updated_on
        )
        VALUES
        (?, ?, ?, ?, ?, ?, ?)";
        let mut statement = connection.prepare(sql).unwrap();

        let params = params![
            entity.id.to_string(),
            entity.name,
            entity.class_id,
            entity.character_id,
            entity.last_gear_score,
            entity.created_on.to_string(),
            entity.updated_on.to_string()
        ];
        statement.execute(params)?;

        Ok(())
    }
}