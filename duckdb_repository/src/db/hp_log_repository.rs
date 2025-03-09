use duckdb::{params, DuckdbConnectionManager};
use r2d2::Pool;
use anyhow::{Ok, Result};

use crate::models::HpLog;

pub struct HpLogRepository {
    pool: Pool<DuckdbConnectionManager>
}

impl HpLogRepository {
    pub fn new(pool: Pool<DuckdbConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn insert(&self, entity: HpLog) -> Result<()> {

        let connection = self.pool.get()?;
        let sql = r"
        INSERT INTO HpLog
        (
            session_id,
            recorded_on,
            value
        )
        VALUES
        (?, ?, ?, ?)";
        let mut statement = connection.prepare(sql).unwrap();

        let params = params![
            entity.session_id.to_string(),
            entity.recorded_on.to_string(),
            entity.value
        ];
        statement.execute(params)?;

        Ok(())
    }
}