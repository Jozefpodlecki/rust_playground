use std::path::Path;

use sqlx::{migrate::{self, MigrateDatabase, Migrator}, Execute, FromRow, Sqlite, SqlitePool};

const DB_URL: &str = "sqlite://sqlite.db";

#[derive(Clone, FromRow, Debug)]
struct User {
    id: i64,
    name: String,
    active: bool,
}

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
   
    let db = SqlitePool::connect(DB_URL).await.unwrap();
    let migrator = Migrator::new(Path::new("./migrations")).await.unwrap();
    let migration_results = migrator.run(&db).await;

    match migration_results {
        Ok(_) => println!("Migration success"),
        Err(error) => {
            panic!("error: {}", error);
        }
    }

    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE name = ?)")
        .bind("john")
        .fetch_one(&db)
        .await
        .unwrap();

    if !exists {
        let result = sqlx::query("INSERT INTO users (name, email) VALUES ($1, $2)")
            .bind("john")
            .bind("test@test.com")
            .execute(&db)
            .await
            .unwrap();
    }

    let user_results = sqlx::query_as::<_, User>("SELECT id, name, active FROM users")
        .fetch_all(&db)
        .await
        .unwrap();

    for user in user_results {
        println!("{:?}", user);
    }
}
