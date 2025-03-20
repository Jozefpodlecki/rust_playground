use std::path::{Path, PathBuf};

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use anyhow::*;
use rusqlite::params;

pub fn create_pool(path: &Path) -> Pool<SqliteConnectionManager> {
    let manager = SqliteConnectionManager::file(&path);
    let pool = r2d2::Pool::builder()
        .build(manager).unwrap();
    
    pool
}

pub fn setup_db(pool: Pool<SqliteConnectionManager>) -> Result<()> {

    let connection = pool.get()?;
    let query = "SELECT 1 FROM sqlite_master WHERE type=? AND name=?";

    let mut statement = connection.prepare(query)?;
    let params = params!["table", "Config"];
    let config_exists = statement.exists(params)?;

    if !config_exists {
        let query = r"
        CREATE TABLE Config
        (
            version NVARCHAR(10),
            created_on DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        ";
        let _ = connection.execute(query, [])?;
    }

    Ok(())
}

pub struct SqliteRepository {
    pool: Pool<SqliteConnectionManager> 
}

impl SqliteRepository {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {

        Self {
            pool,
        }
    }
}

fn main() {
    let path = PathBuf::from("test.db");
    let pool = create_pool(&path);
    setup_db(pool.clone()).unwrap();

    let repository = SqliteRepository::new(pool.clone());
    
}
