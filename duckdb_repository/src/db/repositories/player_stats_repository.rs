use chrono::{TimeZone, Utc};
use duckdb::{params, DuckdbConnectionManager};
use r2d2::Pool;
use anyhow::{Ok, Result};
use uuid::Uuid;

use crate::models::PlayerStats;

pub struct PlayerStatsRepository {
    pool: Pool<DuckdbConnectionManager>
}

impl PlayerStatsRepository {
    pub fn new(pool: Pool<DuckdbConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get_by_player_id(&self, player_id: Uuid) -> Result<Vec<PlayerStats>> {

        let connection = self.pool.get()?;

        let sql = r"
        SELECT
            confrontation_id,
            player_id,
            created_on,
            total_damage_taken,
            total_damage_dealt
        FROM PlayerStats
        WHERE player_id = ?
        ";
    
        let mut statement = connection.prepare(sql)?;
        let params = params![player_id.to_string()];
        let result = statement.query_map(params, Self::map_row)?
            .filter_map(Result::ok)
            .collect();
        
        Ok(result)
    }

    pub fn insert(&self, entity: &PlayerStats) -> Result<()> {

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

    fn map_row(row: &duckdb::Row) -> std::result::Result<PlayerStats, duckdb::Error> {
        let confrontation_id: String = row.get("confrontation_id")?;
        let confrontation_id: Uuid = Uuid::parse_str(&confrontation_id).expect("Invalid id");

        let player_id: String = row.get("player_id")?;
        let player_id = Uuid::parse_str(&player_id).expect("Invalid id");

        let created_on: i64 = row.get("created_on")?;
        let created_on = Utc.timestamp_micros(created_on).unwrap();

        std::result::Result::Ok(PlayerStats {
            created_on,
            player_id,
            confrontation_id,
            total_damage_dealt: row.get("total_damage_dealt")?,
            total_damage_taken: row.get("total_damage_taken")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::{SecondsFormat, Utc};

    use crate::models::PlayerStats;
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

        let expected = PlayerStats {
            confrontation_id: confrontation.id,
            created_on: Utc::now(),
            player_id: player.id,
            total_damage_dealt: 0,
            total_damage_taken: 0
        };

        repository.insert(&expected).unwrap();

        let player_stats = repository.get_by_player_id(player.id).unwrap();
        let actual = player_stats.first().unwrap();

        assert_eq!(actual.player_id, expected.player_id);
        assert_eq!(actual.confrontation_id, expected.confrontation_id);
        assert_eq!(actual.created_on.to_rfc3339_opts(SecondsFormat::Secs, false), expected.created_on.to_rfc3339_opts(SecondsFormat::Secs, false));
        assert_eq!(actual.total_damage_dealt, expected.total_damage_dealt);
        assert_eq!(actual.total_damage_taken, expected.total_damage_taken);
    }
}