use chrono::{Duration, TimeZone, Utc};
use duckdb::{params, DuckdbConnectionManager};
use r2d2::Pool;
use anyhow::{Ok, Result};
use uuid::Uuid;

use crate::{custom_duration::CustomDuration, models::Confrontation};

pub struct ConfrontationRepository {
    pool: Pool<DuckdbConnectionManager>
}

impl ConfrontationRepository {
    pub fn new(pool: Pool<DuckdbConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get_by_min_duration(&self, duration: &CustomDuration)  -> Result<Vec<Confrontation>> {

        let connection = self.pool.get()?;
        let sql = r"
        SELECT
            id,
            created_on,
            raid_id,
            is_cleared,
            total_damage_dealt,
            total_damage_taken,
            duration
        FROM Confrontation
        WHERE duration >= ?
        ";
    
        let mut statement = connection.prepare(sql)?;
        let params = params![duration];
        let rows = statement
            .query_map(params, Self::map_row_to_confrontation)?
            .filter_map(Result::ok)
            .collect();

        Ok(rows)
    }

    pub fn insert(&self, entity: &Confrontation) -> Result<()> {

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
        (?, ?, ?, ?, ?, ?, ?)";
        let mut statement = connection.prepare(sql).unwrap();

        let params = params![
            entity.id.to_string(),
            entity.created_on.to_string(),
            entity.raid_id.to_string(),
            entity.is_cleared,
            entity.total_damage_dealt,
            entity.total_damage_taken,
            entity.duration
        ];
        statement.execute(params)?;

        Ok(())
    }

    fn map_row_to_confrontation(row: &duckdb::Row) -> std::result::Result<Confrontation, duckdb::Error> {
        let id: String = row.get("id")?;
        let id = Uuid::parse_str(&id).expect("Invalid id");

        let created_on: i64 = row.get("created_on")?;
        let created_on = Utc.timestamp_micros(created_on).unwrap();

        let raid_id: String = row.get("raid_id")?;
        let raid_id = Uuid::parse_str(&raid_id).expect("Invalid id");

        std::result::Result::Ok(Confrontation {
            id,
            created_on,
            duration: row.get("duration")?,
            is_cleared: row.get("is_cleared")?,
            raid_id,
            total_damage_dealt: row.get("total_damage_dealt")?,
            total_damage_taken: row.get("total_damage_taken")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::SecondsFormat;

    use crate::db::repositories::utils::TestDb;

    #[test]
    fn test_confrontation() {

        let mut test_db = TestDb::new();
        test_db.setup().unwrap();

        let raid = test_db.create_raid().unwrap();
        let confrontation = test_db.create_confrontation(raid.id).unwrap();

        let repository = test_db.confrontation_repository.unwrap();

        let confrontations = repository.get_by_min_duration(&confrontation.duration).unwrap();
        let actual = confrontations.first().unwrap();

        assert_eq!(actual.id, confrontation.id);
        assert_eq!(actual.created_on.to_rfc3339_opts(SecondsFormat::Secs, false), confrontation.created_on.to_rfc3339_opts(SecondsFormat::Secs, false));
        assert_eq!(actual.total_damage_dealt, confrontation.total_damage_dealt);
        assert_eq!(actual.total_damage_taken, confrontation.total_damage_taken);
    }
}