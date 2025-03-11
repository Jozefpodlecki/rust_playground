use chrono::{DateTime, TimeZone, Utc};
use duckdb::{params, DuckdbConnectionManager};
use r2d2::Pool;
use anyhow::{Ok, Result};
use uuid::Uuid;

use crate::models::Player;

pub struct PlayerRepository {
    pool: Pool<DuckdbConnectionManager>
}

impl PlayerRepository {
    pub fn new(pool: Pool<DuckdbConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn exists(&self, name: &str) -> Result<bool> {

        let connection = self.pool.get()?;

        let sql = r"
        SELECT
            EXISTS (SELECT 1 FROM Player WHERE name = ?)
        ";
    
        let mut statement = connection.prepare(sql)?;
        let params = [name];
        let result = statement.query_row(params, |row| row.get(0))?;
        
        Ok(result)
    }

    pub fn get_by_name(&self, name: &str) -> Result<Player> {
        let connection = self.pool.get()?;
    
        let sql = r"
        SELECT
            id,
            name,
            class_id,
            character_id,
            last_gear_score,
            created_on,
            updated_on
        FROM Player
        WHERE name = ?
        ";
    
        let mut statement = connection.prepare(sql)?;
        let params = [name];
        let row = statement.query_row(params, Self::map_row_to_player)?;
    
        Ok(row)
    }

    pub fn get_by_character_id(&self, character_id: u64) -> Result<Player> {
        let connection = self.pool.get()?;
    
        let sql = r"
        SELECT
            id,
            name,
            class_id,
            character_id,
            last_gear_score,
            created_on,
            updated_on
        FROM Player
        WHERE character_id = ?
        ";
    
        let mut statement = connection.prepare(sql)?;
        let params = [character_id];
        let row = statement.query_row(params, Self::map_row_to_player)?;
    
        Ok(row)
    }

    pub fn insert(&self, entity: &Player) -> Result<()> {

        let connection = self.pool.get()?;
        let sql = "
        INSERT INTO Player
        (
            id,
            name,
            class_id,
            character_id,
            last_gear_score,
            created_on,
            updated_on
        )
        VALUES
        (?, ?, ?, ?, ?, ?, ?)";
        let mut statement = connection.prepare(sql).unwrap();

        let params = params![
            entity.id.to_string(),
            entity.name,
            entity.class_id,
            entity.character_id,
            entity.last_gear_score,
            entity.created_on.to_string(),
            entity.updated_on.to_string()
        ];
        let _ = statement.execute(params)?;

        Ok(())
    }

    fn map_row_to_player(row: &duckdb::Row) -> std::result::Result<Player, duckdb::Error> {
        let id: String = row.get("id")?;
        let id = Uuid::parse_str(&id).expect("Invalid id");

        let created_on: i64 = row.get("created_on")?;
        let created_on = Utc.timestamp_micros(created_on).unwrap();

        let updated_on: i64 = row.get("updated_on")?;
        let updated_on = Utc.timestamp_micros(updated_on).unwrap();

        std::result::Result::Ok(Player {
            id,
            name: row.get("name")?,
            class_id: row.get("class_id")?,
            character_id: row.get("character_id")?,
            last_gear_score: row.get("last_gear_score")?,
            created_on,
            updated_on,
        })
    }
}


#[cfg(test)]
mod tests {
    use crate::db::repositories::utils::TestDb;

    #[test]
    fn test_insert_player_and_get() {

        let mut test_db = TestDb::new();
        test_db.setup().unwrap();
        
        let player = test_db.create_player().unwrap();
        let repository = test_db.player_repository.unwrap();

        let fetched_by_id = repository.get_by_character_id(player.character_id).unwrap();
        assert_eq!(fetched_by_id.name, player.name);
        assert_eq!(fetched_by_id.character_id, player.character_id);
        assert_eq!(fetched_by_id.last_gear_score, player.last_gear_score);

        let fetched_by_name = repository.get_by_name(&player.name).unwrap();
        assert_eq!(fetched_by_name.character_id, player.character_id);
        assert_eq!(fetched_by_name.name, player.name);

        let exists = repository.exists(&player.name).unwrap();
        assert!(exists);

        let does_not_exist = repository.exists("NonExistingPlayer").unwrap();
        assert!(!does_not_exist);

        let result = repository.get_by_name("NonExistingPlayer");
        assert!(result.is_err());

        let result = repository.get_by_character_id(9999);
        assert!(result.is_err());

    }
}