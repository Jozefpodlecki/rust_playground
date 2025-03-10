use chrono::{DateTime, Utc};
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
        let row = statement.query_row(params, |row| {
            let id: String = row.get("id")?;
            let id = Uuid::parse_str(&id).expect("Invalid id");
            let created_on: String = row.get("created_on")?;
            let created_on = created_on.parse::<DateTime<Utc>>().expect("Invalid created_on");

            let updated_on: String = row.get("updated_on")?;
            let updated_on = updated_on.parse::<DateTime<Utc>>().expect("Invalid updated_on");

            std::result::Result::Ok(Player {
                id,
                name: row.get("name")?,
                class_id: row.get("class_id")?,
                character_id: row.get("character_id")?,
                last_gear_score: row.get("last_gear_score")?,
                created_on,
                updated_on,
            })
        })?;
    
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
        let row = statement.query_row(params, |row| {
            let id: String = row.get("id")?;
            let id = Uuid::parse_str(&id).expect("Invalid id");
            let created_on: String = row.get("created_on")?;
            println!("{:?}", created_on);
            let created_on = created_on.parse::<DateTime<Utc>>().expect("Invalid created_on");

            let updated_on: String = row.get("updated_on")?;
            let updated_on = updated_on.parse::<DateTime<Utc>>().expect("Invalid updated_on");

            std::result::Result::Ok(Player {
                id,
                name: row.get("name")?,
                class_id: row.get("class_id")?,
                character_id: row.get("character_id")?,
                last_gear_score: row.get("last_gear_score")?,
                created_on,
                updated_on,
            })
        })?;
    
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
        statement.execute(params)?;

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::db::migration::MigrationRunner;

    use super::*;
    use duckdb::DuckdbConnectionManager;
    use r2d2::Pool;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_insert_player_and_get() {

        let manager = DuckdbConnectionManager::memory().unwrap();
        let pool = Pool::new(manager).unwrap();
        let repository = PlayerRepository::new(pool.clone());
        let migration_runner = MigrationRunner::new(pool.clone());
        migration_runner.run("0.1.0").unwrap();

        let character_id = 1;

        let player = Player {
            id: Uuid::now_v7(),
            updated_on: Utc::now(),
            created_on: Utc::now(),
            character_id,
            class_id: 1,
            last_gear_score: 1670.0,
            name: "Test".into(),
        };

        repository.insert(&player).unwrap();

        let player = repository.get_by_character_id(character_id).unwrap();

        println!("{:?}", player);
    }
}