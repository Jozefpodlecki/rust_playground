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
        Ok(vec![])
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

#[cfg(test)]
mod tests {
    use chrono::{Duration, SubsecRound, Utc};

    use crate::{db::repositories::utils::{setup_test_database, TestDb}, models::{Confrontation, HpLog}};

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