use std::path::Path;

use sqlx::{migrate::{self, MigrateDatabase, Migrator}, Sqlite, SqlitePool};

const DB_URL: &str = "sqlite://sqlite.db";

#[tokio::main]
async fn main() {

    let database_exists = Sqlite::database_exists(DB_URL).await.unwrap_or(false);

    if database_exists {
        println!("Database already exists");
    }
    else {
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => println!("Create db success"),
            Err(error) => panic!("error: {}", error),
        }
    }
   
    let migrator = Migrator::new(Path::new("./migrations")).await.unwrap();
    // Migrator::run(&self, migrator).await.unwrap();
    // migrator.run().await.unwrap();

    let db = SqlitePool::connect(DB_URL).await.unwrap();
}
