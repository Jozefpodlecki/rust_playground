mod file_database;
use file_database::open_database;
use file_database::save_database;
use std::time::{SystemTime};
use chrono::prelude::{DateTime, Utc};
use std::collections::HashMap;

fn main() {
    let file_name = "database.json".to_string();
    let datetime = Utc::now();
    let mut database = open_database(file_name.clone());
    let formatted_now = format!("{}", datetime.format("%Y-%m-%d %H:%M:%S"));

    database.entry("createdAt".to_string()).or_insert(formatted_now.clone());
    database.entry("modifiedAt".to_string()).or_insert(formatted_now.clone());
    *database.get_mut("modifiedAt").unwrap() = formatted_now;

    let mut items: Vec<String> = vec![];
    items.push("test".to_string());

    let serialized_list = serde_json::to_string(&items).unwrap();

    database.entry("items".to_string()).or_insert(serialized_list);

    save_database(file_name, database);
}
