use chrono::Utc;
use duckdb::{params, Connection, DuckdbConnectionManager, Result};
use uuid::Uuid;

fn main() {
    let manager = DuckdbConnectionManager::file("db.duckdb").unwrap();
    
    let pool = r2d2::Pool::builder()
        .build(manager)
        .unwrap();

    let sql = r"
    CREATE TABLE Config(
        version TEXT PRIMARY KEY
    )
    ";

    let connection = pool.get().unwrap();
    connection.execute_batch(sql).unwrap();

    let sql = r"
    CREATE TABLE Player(
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        created_on TIMESTAMP NOT NULL
    )
    ";

    connection.execute_batch(sql).unwrap();

    let new_id = Uuid::now_v7().to_string();
    let player_name = "Alice";
    let created_on = Utc::now().to_rfc3339();

    connection.execute(
        "INSERT INTO Player (id, name, created_on) VALUES (?, ?, ?)",
        params![new_id, player_name, created_on],
    ).unwrap();
}
