use chrono::{TimeZone, Utc};
use duckdb::{arrow::array::{GenericListArray, ListArray}, params, types::{FromSql, Value}, DuckdbConnectionManager};
use r2d2::Pool;
use anyhow::{Ok, Result};
use uuid::Uuid;

use crate::models::Raid;

pub struct RaidRepository {
    pool: Pool<DuckdbConnectionManager>
}

impl RaidRepository {
    pub fn new(pool: Pool<DuckdbConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get_by_names_and_gate(&self, name: &str, sub_name: Option<&String>, gate: u8) -> Result<Raid> {
        let connection = self.pool.get()?;
    
        let sql = r"
        SELECT
            id,
            created_on,
            name,
            sub_name,
            gate,
            zone_ids
        FROM Raid
        WHERE name = ?
            AND (sub_name = ? OR sub_name IS NULL)
            AND gate = ?
        ";
    
        let mut statement = connection.prepare(sql)?;
        let params = params![name, sub_name, gate];
        let row = statement.query_row(params, Self::map_row_to_raid)?;
    
        Ok(row)
    }

    pub fn insert(&self, entity: &Raid) -> Result<()> {

        let connection = self.pool.get()?;
        let sql = "
        INSERT INTO Raid
        (
            id,
            created_on,
            name,
            sub_name,
            gate,
            zone_ids
        )
        VALUES
        (?, ?, ?, ?, ?, ?)";
        let mut statement = connection.prepare(sql).unwrap();

        let params = params![
            entity.id.to_string(),
            entity.created_on.to_string(),
            entity.name,
            entity.sub_name,
            entity.gate,
            format!("{:?}", entity.zone_ids),
        ];
        
        statement.execute(params)?;

        Ok(())
    }

    fn map_row_to_raid(row: &duckdb::Row) -> std::result::Result<Raid, duckdb::Error> {
        let id: String = row.get("id")?;
        let id = Uuid::parse_str(&id).expect("Invalid id");

        let created_on: i64 = row.get("created_on")?;
        let created_on = Utc.timestamp_micros(created_on).unwrap();

        let sql_zone_ids: Value = row.get("zone_ids")?;
        let mut zone_ids: Vec<u32> = vec![];

        if let Value::List(array) = sql_zone_ids {
            for value in array {
                if let Value::UInt(value) = value {
                    zone_ids.push(value);
                }
            }   
        }

        std::result::Result::Ok(Raid {
            id,
            created_on,
            gate: row.get("gate")?,
            name: row.get("name")?,
            sub_name: row.get("sub_name")?,
            zone_ids,
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::SecondsFormat;

    use crate::db::repositories::utils::TestDb;

    #[test]
    fn test_raid() {

        let mut test_db = TestDb::new();
        test_db.setup().unwrap();

        let raid = test_db.create_raid().unwrap();
        let repository = test_db.raid_repository.unwrap();

        let fetched_raid = repository.get_by_names_and_gate(&raid.name, raid.sub_name.as_ref(), raid.gate).unwrap();

        assert_eq!(fetched_raid.id, raid.id);
        assert_eq!(fetched_raid.created_on.to_rfc3339_opts(SecondsFormat::Secs, false), raid.created_on.to_rfc3339_opts(SecondsFormat::Secs, false));
        assert_eq!(fetched_raid.name, raid.name);
        assert_eq!(fetched_raid.sub_name, raid.sub_name);
        assert_eq!(fetched_raid.gate, raid.gate);
        assert_eq!(fetched_raid.zone_ids, raid.zone_ids);
    }
}