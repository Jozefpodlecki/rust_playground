use chrono::{DateTime, TimeZone, Utc};
use duckdb::{params, DuckdbConnectionManager, OptionalExt};
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

    pub fn get_by_id(&self, id: Uuid) -> Result<Option<HpSession>> {

        let connection = self.pool.get()?;

        let sql = r"
        SELECT
            id,
            npc_id,
            confrontation_id,
            started_on,
            ended_on
        FROM HpSession
        WHERE id = ?
        ";
    
        let mut statement = connection.prepare(sql)?;
        let params = params![id.to_string()];
        let result = statement.query_row(params, Self::map_row)
            .optional()?;
        
        Ok(result)

    }

    pub fn insert(&self, entity: &HpSession) -> Result<()> {

        let connection = self.pool.get()?;
        let sql = r"
        INSERT INTO HpSession
        (
            id,
            npc_id,
            confrontation_id,
            started_on
        )
        VALUES
        (?, ?, ?, ?)";
        let mut statement = connection.prepare(sql).unwrap();

        let params = params![
            entity.id.to_string(),
            entity.npc_id.to_string(),
            entity.confrontation_id.to_string(),
            entity.started_on.to_string(),
        ];
        statement.execute(params)?;

        Ok(())
    }

    fn map_row(row: &duckdb::Row) -> std::result::Result<HpSession, duckdb::Error> {
        let id: String = row.get("id")?;
        let id = Uuid::parse_str(&id).expect("Invalid id");

        let npc_id: String = row.get("npc_id")?;
        let npc_id = Uuid::parse_str(&npc_id).expect("Invalid id");

        let started_on: i64 = row.get("started_on")?;
        let started_on = Utc.timestamp_micros(started_on).unwrap();

        let ended_on: Option<i64> = row.get("ended_on")?;
        let ended_on = ended_on.map(|value| Utc.timestamp_micros(value).unwrap());

        let confrontation_id: String = row.get("confrontation_id")?;
        let confrontation_id = Uuid::parse_str(&confrontation_id).expect("Invalid id");

        std::result::Result::Ok(HpSession {
            id,
            confrontation_id,
            started_on,
            ended_on,
            npc_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::{SecondsFormat, Utc};

    use crate::db::repositories::utils::TestDb;

    #[test]
    fn test_hp_session() {

        let mut test_db = TestDb::new();
        test_db.setup().unwrap();

        let raid = test_db.create_raid().unwrap();
        let boss = test_db.create_npc(raid.id).unwrap();
        let confrontation = test_db.create_confrontation(raid.id).unwrap();
        let hp_session = test_db.create_hp_session(confrontation.id, boss.id).unwrap();

        let repository = test_db.hp_session_repository.unwrap();

        repository.update(hp_session.id, Utc::now()).unwrap();

        let actual = repository.get_by_id(hp_session.id).unwrap().unwrap();

        assert_eq!(actual.id, hp_session.id);
        assert_eq!(actual.started_on.to_rfc3339_opts(SecondsFormat::Secs, false), hp_session.started_on.to_rfc3339_opts(SecondsFormat::Secs, false));
        assert!(actual.ended_on.is_some());
        assert_eq!(actual.npc_id, hp_session.npc_id);
    }
}