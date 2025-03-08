use std::{env, fs};

use duckdb::{Connection, DuckdbConnectionManager};
use r2d2::Pool;
use anyhow::{Ok, Result};

pub struct MigrationRunner {
    pool: Pool<DuckdbConnectionManager>
}

impl MigrationRunner {
    pub fn new(pool: Pool<DuckdbConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn run(&self, version: &str) -> Result<()> {

        let connection = self.pool.get()?;

        if Self::table_exists(&connection, "Config")? {
            // execute here 0.0.1.sql maybe?

            let executable_path = env::current_exe()?;
            let executable_directory = executable_path.parent().unwrap();
            let migrations_directory = executable_directory.join("migrations");
            
            let mut files: Vec<_> = fs::read_dir(migrations_directory)
                .unwrap()
                .map(|e| e.unwrap().path())
                .filter(|p| p.extension().map_or(false, |ext| ext == "sql"))
                .collect();
            
            files.sort();

            return Ok(())
        }

        Ok(())
    }

    fn table_exists(connection: &Connection, name: &str) -> Result<bool> {
        let sql = r"
        SELECT
            EXISTS (SELECT 1 FROM duckdb_tables WHERE table_name = ?)
        ";
    
        let mut statement = connection.prepare(sql).unwrap();
        let result = statement.query_row([name], |row| row.get(0))?;
        
        Ok(result)
    }
}