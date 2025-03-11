pub struct AppConfig {
    pub version: String,
    pub database_name: String,
}

impl AppConfig {
    pub fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            database_name: "db.duckdb".to_string(),
        }
    }
}