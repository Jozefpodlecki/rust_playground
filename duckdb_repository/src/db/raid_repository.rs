use duckdb::{params, DuckdbConnectionManager};
use r2d2::Pool;
use anyhow::{Ok, Result};

use crate::models::Raid;

pub struct RaidRepository {
    pool: Pool<DuckdbConnectionManager>
}

impl RaidRepository {
    pub fn new(pool: Pool<DuckdbConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn insert(&self, entity: Raid) -> Result<()> {

        let connection = self.pool.get()?;
        let sql = "
        INSERT INTO Raid
        (
            id,
            created_on,
            name,
            sub_name
            gate,
            zone_ids
        )
        VALUES
        (?, ?, ?, ?, ?)";
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
}

#[cfg(test)]
mod tests {
    use crate::db::migration::MigrationRunner;

    use super::*;
    use duckdb::{params, DuckdbConnectionManager};
    use r2d2::Pool;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_insert_raid() {

        let manager = DuckdbConnectionManager::memory().unwrap();
        let pool = Pool::new(manager).unwrap();
        let repository = RaidRepository::new(pool.clone());
        let migration_runner = MigrationRunner::new(pool.clone());

        migration_runner.run("0.1.0").unwrap();

        let raid = Raid {
            id: Uuid::now_v7(),
            created_on: Utc::now(),
            name: "Legion Raid".to_string(),
            sub_name: Some("Vykas".to_string()),
            gate: 3,
            zone_ids: vec![101, 102, 103],
        };

        repository.insert(raid).unwrap();

        // // Call the insert function
        // let result = repo.insert(raid.clone());
        // assert!(result.is_ok());

        // // Verify the data in the database
        // let mut stmt = connection.prepare("SELECT * FROM Raid").unwrap();
        // let mut rows = stmt.query([]).unwrap();

        // let row = rows.next().unwrap().unwrap();
        // let id: String = row.get(0).unwrap();
        // let created_on: String = row.get(1).unwrap();
        // let name: String = row.get(2).unwrap();
        // let sub_name: String = row.get(3).unwrap();
        // let gate: i32 = row.get(4).unwrap();
        // let zone_ids: String = row.get(5).unwrap();

        // assert_eq!(id, raid.id.to_string());
        // assert_eq!(created_on, raid.created_on.to_string());
        // assert_eq!(name, raid.name);
        // assert_eq!(sub_name, raid.sub_name);
        // assert_eq!(gate, raid.gate);
        // assert_eq!(zone_ids, format!("{:?}", raid.zone_ids));
    }
}