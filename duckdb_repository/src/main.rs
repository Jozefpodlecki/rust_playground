mod db;
mod models;

use std::{env, fs};

use chrono::Utc;
use db::{migration::MigrationRunner, repository::{self, PlayerRepository}};
use duckdb::DuckdbConnectionManager;
use models::Player;
use uuid::Uuid;

fn main() {

    let executable_path = env::current_exe().unwrap();
    let executable_directory = executable_path.parent().unwrap();
    let migrations_directory = executable_directory.join("migrations");
    
    let mut files: Vec<_> = fs::read_dir(migrations_directory)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().map_or(false, |ext| ext == "sql"))
        .collect();
    
    files.sort();

    for file in files {
        println!("{:?}", file.to_str());
    }

    return;

    let version = env!("CARGO_PKG_VERSION");
    let manager = DuckdbConnectionManager::file("db.duckdb").unwrap();
    
    let pool = r2d2::Pool::builder()
        .build(manager)
        .unwrap();

    
    let migration_runner = MigrationRunner::new(pool.clone());
    let player_repository = PlayerRepository::new(pool.clone());

    migration_runner.run(version).unwrap();
   
    let player = Player {
        id: Uuid::now_v7(),
        name: "Alice".into(),
        created_on: Utc::now()
    };

    player_repository.insert(player).unwrap();
}
