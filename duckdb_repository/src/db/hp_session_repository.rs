use chrono::{DateTime, Utc};
use duckdb::{params, DuckdbConnectionManager};
use r2d2::Pool;
use anyhow::{Ok, Result};
use uuid::Uuid;

use crate::models::HpSession;

pub struct HpSessionRepository {
    pool: Pool<DuckdbConnectionManager>
}

impl HpSessionRepository {
    pub fn new(pool: Pool<DuckdbConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn update(&self, session_id: Uuid, ended_on: DateTime<Utc>) -> Result<()> {
        let connection = self.pool.get()?;

        let sql = r"
        UPDATE HpSession
        SET ended_on = ?
        WHERE id = ?";
        
        let mut statement = connection.prepare(sql)?;

        let params = params![
            ended_on.to_string(),
            session_id.to_string(),
        ];
        statement.execute(params)?;

        Ok(())
    }

    pub fn insert(&self, entity: HpSession) -> Result<()> {

        let connection = self.pool.get()?;
        let sql = r"
        INSERT INTO HpSession
        (
            id,
            entity_id,
            raid_id,
            started_on
        )
        VALUES
        (?, ?, ?, ?)";
        let mut statement = connection.prepare(sql).unwrap();

        let params = params![
            entity.id.to_string(),
            entity.entity_id.to_string(),
            entity.raid_id.to_string(),
            entity.started_on.to_string(),
        ];
        statement.execute(params)?;

        Ok(())
    }
}