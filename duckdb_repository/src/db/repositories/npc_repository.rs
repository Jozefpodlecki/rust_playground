use chrono::{TimeZone, Utc};
use duckdb::{params, DuckdbConnectionManager, OptionalExt};
use r2d2::Pool;
use anyhow::{Ok, Result};
use uuid::Uuid;

use crate::models::Npc;

pub struct NpcRepository {
    pool: Pool<DuckdbConnectionManager>
}

impl NpcRepository {
    pub fn new(pool: Pool<DuckdbConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get_by_name_and_raid(&self, name: &str, raid_id: Uuid) -> Result<Vec<Npc>> {

        let connection = self.pool.get()?;

        let sql = r"
        SELECT
            id,
            created_on,
            name,
            npc_id,
            npc_type,
            raid_id
        FROM Npc
        WHERE name = ?
            AND raid_id = ?
        ";
    
        let mut statement = connection.prepare(sql)?;
        let params = params![name, raid_id.to_string()];
        let result = statement.query_map(params, Self::map_row_to_npc)?
            .filter_map(Result::ok)
            .collect();
        
        Ok(result)
    }

    pub fn insert(&self, entity: &Npc) -> Result<()> {

        let connection = self.pool.get()?;
        let sql = "
        INSERT INTO Npc
        (
            id,
            created_on,
            name,
            npc_id,
            npc_type,
            raid_id
        )
        VALUES
        (?, ?, ?, ?, ?, ?)";
        let mut statement = connection.prepare(sql).unwrap();

        let params = params![
            entity.id.to_string(),
            entity.created_on.to_string(),
            entity.name,
            entity.npc_id,
            entity.npc_type,
            entity.raid_id.to_string(),
        ];
        statement.execute(params)?;

        Ok(())
    }

    fn map_row_to_npc(row: &duckdb::Row) -> std::result::Result<Npc, duckdb::Error> {
        let id: String = row.get("id")?;
        let id = Uuid::parse_str(&id).expect("Invalid id");

        let created_on: i64 = row.get("created_on")?;
        let created_on = Utc.timestamp_micros(created_on).unwrap();

        let raid_id: String = row.get("raid_id")?;
        let raid_id = Uuid::parse_str(&raid_id).expect("Invalid id");

        std::result::Result::Ok(Npc {
            id,
            created_on,
            name: row.get("name")?,
            npc_id: row.get("npc_id")?,
            npc_type: row.get("npc_type")?,
            raid_id
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::SecondsFormat;

    use crate::db::repositories::utils::TestDb;

    #[test]
    fn test_insert_npc_and_get() {

        let mut test_db = TestDb::new();
        test_db.setup().unwrap();

        let raid = test_db.create_raid().unwrap();
        let npc = test_db.create_npc(raid.id).unwrap();

        let repository = test_db.npc_repository.unwrap();

        let npcs = repository.get_by_name_and_raid(&npc.name, raid.id).unwrap();
        let actual = npcs.first().unwrap();

        assert_eq!(actual.id, npc.id);
        assert_eq!(actual.created_on.to_rfc3339_opts(SecondsFormat::Secs, false), npc.created_on.to_rfc3339_opts(SecondsFormat::Secs, false));
        assert_eq!(actual.name, npc.name);
        assert_eq!(actual.npc_id, npc.npc_id);
        assert_eq!(actual.npc_type, npc.npc_type);
        assert_eq!(actual.raid_id, npc.raid_id);
        
    }
}