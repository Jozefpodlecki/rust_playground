use duckdb::{params, DuckdbConnectionManager};
use r2d2::Pool;
use anyhow::{Ok, Result};

use crate::models::PlayerStats;

pub struct PlayerStatsRepository {
    pool: Pool<DuckdbConnectionManager>
}

impl PlayerStatsRepository {
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

    pub fn insert(&self, entity: PlayerStats) -> Result<()> {

        let connection = self.pool.get()?;
        let sql = "
        INSERT INTO PlayerStats
        (
            confrontation_id,
            player_id,
            created_on,
            total_damage_taken,
            total_damage_dealt
        )
        VALUES
        (?, ?, ?, ?, ?)";
        let mut statement = connection.prepare(sql).unwrap();

        let params = params![
            entity.confrontation_id.to_string(),
            entity.player_id.to_string(),
            entity.created_on.to_string(),
            entity.total_damage_taken,
            entity.total_damage_dealt
        ];
        statement.execute(params)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::{db::repositories::utils::setup_test_database, models::PlayerStats};
    use crate::db::repositories::utils::TestDb;
    use super::PlayerStatsRepository;

    #[test]
    fn test_player_stats() {

        let mut test_db = TestDb::new();
        test_db.setup().unwrap();

        let pool = test_db.pool.clone();
        let raid = test_db.create_raid().unwrap();
        let confrontation = test_db.create_confrontation(raid.id).unwrap();
        let player = test_db.create_player().unwrap();

        let repository = PlayerStatsRepository::new(pool.clone());

        let player_stats = PlayerStats {
            confrontation_id: confrontation.id,
            created_on: Utc::now(),
            player_id: player.id,
            total_damage_dealt: 0,
            total_damage_taken: 0
        };

        repository.insert(player_stats).unwrap()
    }
}