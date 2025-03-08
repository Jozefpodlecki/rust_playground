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

    pub fn insert(&self, player: Player) -> Result<()> {

        let connection = self.pool.get()?;
        let sql = "INSERT INTO Player (id, name, created_on) VALUES (?, ?, ?)";
        let mut statement = connection.prepare(sql).unwrap();

        let params = params![player.id.to_string(), player.name, player.created_on.to_string()];
        statement.execute(params)?;

        Ok(())
    }
}