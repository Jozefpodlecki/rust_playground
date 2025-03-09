use duckdb::{params, DuckdbConnectionManager};
use r2d2::Pool;
use anyhow::{Ok, Result};

use crate::models::Npc;

pub struct NpcRepository {
    pool: Pool<DuckdbConnectionManager>
}

impl NpcRepository {
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

    pub fn insert(&self, entity: Npc) -> Result<()> {

        let connection = self.pool.get()?;
        let sql = "
        INSERT INTO Npc
        (
            id,
            created_on,
            name,
            npc_type_id,
            raid_id
        )
        VALUES
        (?, ?, ?, ?, ?)";
        let mut statement = connection.prepare(sql).unwrap();

        let params = params![
            entity.id.to_string(),
            entity.created_on.to_string(),
            entity.name,
            entity.npc_type_id,
            entity.raid_id.to_string(),
        ];
        statement.execute(params)?;

        Ok(())
    }
}