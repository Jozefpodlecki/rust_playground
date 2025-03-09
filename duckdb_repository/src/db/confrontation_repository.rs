use duckdb::{params, DuckdbConnectionManager};
use r2d2::Pool;
use anyhow::{Ok, Result};

use crate::models::Confrontation;

pub struct ConfrontationRepository {
    pool: Pool<DuckdbConnectionManager>
}

impl ConfrontationRepository {
    pub fn new(pool: Pool<DuckdbConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn insert(&self, entity: Confrontation) -> Result<()> {

        let connection = self.pool.get()?;
        let sql = r"
        INSERT INTO Confrontation
        (
            id,
            created_on,
            raid_id,
            is_cleared,
            total_damage_dealt,
            total_damage_taken,
            duration
        )
        VALUES
        (?, ?, ?, ?)";
        let mut statement = connection.prepare(sql).unwrap();

        let params = params![
            entity.id.to_string(),
            entity.created_on.to_string(),
            entity.raid_id.to_string(),
            entity.is_cleared,
            entity.total_damage_dealt,
            entity.total_damage_taken
        ];
        statement.execute(params)?;

        Ok(())
    }
}