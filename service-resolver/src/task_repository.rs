use std::error::Error;

use duckdb::{params, DuckdbConnectionManager};
use r2d2::Pool;

pub trait TaskRepository: Send + Sync {
    fn insert(&mut self) -> Result<(), Box<dyn Error>>;
}

pub struct DefaultTaskRepository {
    pool: Pool<DuckdbConnectionManager>
}

impl TaskRepository for DefaultTaskRepository {
    fn insert(&mut self) -> Result<(), Box<dyn Error>> {
        let connection = self.pool.get()?;
        let query = "INSERT INTO task
        (id, title, done)
        VALUES
        (?, ?, ?);";

        let params = params![];
        let result = connection.execute(query, params)?;
        Ok(())
    }
}

impl DefaultTaskRepository {
    pub fn new(pool: Pool<DuckdbConnectionManager>) -> Self {
        Self {
            pool,
        }
    }
}