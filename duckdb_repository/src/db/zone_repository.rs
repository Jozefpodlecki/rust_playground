use duckdb::{params, DuckdbConnectionManager};
use r2d2::Pool;
use anyhow::{Ok, Result};

use crate::models::{Raid, Zone};

pub struct ZoneRepository {
    pool: Pool<DuckdbConnectionManager>
}

impl ZoneRepository {
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

    pub fn insert(&self, entity: Zone) -> Result<()> {

        let connection = self.pool.get()?;
        let sql = "
        INSERT INTO Raid
        (
            id,
            created_on,
            name,
        )
        VALUES
        (?, ?, ?)";
        let mut statement = connection.prepare(sql).unwrap();

        let params = params![
            entity.id,
            entity.name,
        ];
        statement.execute(params)?;

        Ok(())
    }
}