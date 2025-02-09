use std::error::Error;

use duckdb::DuckdbConnectionManager;
use r2d2::Pool;

pub trait DatabaseMigraiton: Send + Sync {
    fn run(&mut self) -> Result<(), Box<dyn Error>>;
}

pub struct DefaultDatabaseMigraiton {
    pool: Pool<DuckdbConnectionManager>
}

impl DatabaseMigraiton for DefaultDatabaseMigraiton {
    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let connection = self.pool.get()?;

        connection.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            )",
            [],
        )?;

        Ok(())
    }
}

impl DefaultDatabaseMigraiton {
    pub fn new(pool: Pool<DuckdbConnectionManager>) -> Self {
        Self {
            pool
        }
    }
}