
use chrono::Utc;
use duckdb::{params, DuckdbConnectionManager};
use r2d2::Pool;
use anyhow::{Ok, Result};

pub struct ConfigRepository {
    pool: Pool<DuckdbConnectionManager>,
}

impl ConfigRepository {
    pub fn new(pool: Pool<DuckdbConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn update_config_table(&self, version: &str, migration: &str) -> Result<()> {
        let connection = self.pool.get()?;

        let sql = r"
        INSERT INTO Config
        (version, last_migration, updated_on)
        VALUES
        (?, ?, ?)
        ON CONFLICT(version) DO UPDATE SET
            last_migration = excluded.last_migration,
            updated_on = excluded.updated_on
        ";

        let current_timestamp = Utc::now().to_string();
        let mut statement = connection.prepare(sql)?;
        let params = params![version, migration, current_timestamp];
        let _ = statement.execute(params)?;

        Ok(())
    }

    pub fn get_last_applied_migration(&self) -> Result<Option<String>> {
        let connection = self.pool.get()?;

        let sql = "SELECT last_migration FROM Config LIMIT 1";
        let mut statement = connection.prepare(sql)?;
        let result = statement.query_row([], |row| row.get(0)).ok();
        Ok(result)
    }

    pub fn table_exists(&self, name: &str) -> Result<bool> {
        let connection = self.pool.get()?;

        let sql = r"
        SELECT
            EXISTS (SELECT 1 FROM duckdb_tables WHERE table_name = ?)
        ";
        
        let mut statement = connection.prepare(sql)?;
        let params = [name];
        let result = statement.query_row(params, |row| row.get(0))?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::db::repositories::utils::setup_test_database;

    use super::ConfigRepository;

    #[test]
    fn test_config() {

        let pool = setup_test_database().unwrap();
        let config_repository = ConfigRepository::new(pool.clone());

        let config_exists = config_repository.table_exists("Config").unwrap();
        assert!(config_exists);

        let last_migration = config_repository.get_last_applied_migration().unwrap();
        assert_ne!(last_migration, None);

        config_repository.update_config_table("0.1.1", "3_test.sql").unwrap();

        let last_migration = config_repository.get_last_applied_migration().unwrap();
        assert_eq!(last_migration, Some("3_test.sql".to_string()));
    }
}