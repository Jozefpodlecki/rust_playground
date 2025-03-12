use chrono::{TimeZone, Utc};
use duckdb::{params, DuckdbConnectionManager};
use r2d2::Pool;
use anyhow::{Ok, Result};
use uuid::Uuid;

use crate::models::HpLog;

pub struct HpLogRepository {
    pool: Pool<DuckdbConnectionManager>
}

impl HpLogRepository {
    pub fn new(pool: Pool<DuckdbConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get_by_session_id(&self, session_id: Uuid) -> Result<Vec<HpLog>> {

        let connection = self.pool.get()?;

        let sql = r"
        SELECT
            session_id,
            recorded_on,
            value
        FROM HpLog
        WHERE session_id = ?
        ";
    
        let mut statement = connection.prepare(sql)?;
        let params = params![session_id.to_string()];
        let result = statement.query_map(params, Self::map_row)?
            .filter_map(Result::ok)
            .collect();

        Ok(result)
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
        (?, ?, ?)";
        let mut statement = connection.prepare(sql).unwrap();

        let params = params![
            entity.session_id.to_string(),
            entity.recorded_on.to_string(),
            entity.value
        ];
        statement.execute(params)?;

        Ok(())
    }

    fn map_row(row: &duckdb::Row) -> std::result::Result<HpLog, duckdb::Error> {
        let recorded_on: i64 = row.get("recorded_on")?;
        let recorded_on = Utc.timestamp_micros(recorded_on).unwrap();

        let session_id: String = row.get("session_id")?;
        let session_id = Uuid::parse_str(&session_id).expect("Invalid id");

        let value: i64 = row.get("value")?;

        std::result::Result::Ok(HpLog {
            session_id,
            recorded_on,
            value
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, SubsecRound, Utc};

    use crate::{db::repositories::utils::TestDb, models::HpLog};

    use super::HpLogRepository;

    #[test]
    fn test_log_crud() {

        let mut test_db = TestDb::new();
        test_db.setup().unwrap();

        let raid = test_db.create_raid().unwrap();
        let boss = test_db.create_npc(raid.id).unwrap();
        let confrontation = test_db.create_confrontation(raid.id).unwrap();
        let hp_session = test_db.create_hp_session(confrontation.id, boss.id).unwrap();

        let repository = HpLogRepository::new(test_db.pool.clone());

        let hp_log = HpLog {
            session_id: hp_session.id,
            recorded_on: Utc::now(),
            value: 900
        };

        repository.insert(hp_log).unwrap();

        let hp_log = HpLog {
            session_id: hp_session.id,
            recorded_on: Utc::now() - Duration::seconds(1),
            value: 1000
        };

        repository.insert(hp_log).unwrap();

        let hp_logs = repository.get_by_session_id(hp_session.id).unwrap();
        assert_eq!(hp_logs.len(), 2, "Should populate table with 2 logs for given session");
    }
}