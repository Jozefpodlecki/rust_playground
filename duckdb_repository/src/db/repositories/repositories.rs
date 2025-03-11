use crate::{abstractions::FileSystem, app_config::AppConfig, db::migration::MigrationRunner};
use duckdb::DuckdbConnectionManager;

use super::*;

pub struct Repositories {
    pub player_repository: PlayerRepository,
    pub hp_log_repository: HpLogRepository,
    pub hp_session_repository: HpSessionRepository,
    pub confrontation_repository: ConfrontationRepository,
    pub player_stats_repository: PlayerStatsRepository,
    pub npc_repository: NpcRepository,
    pub raid_repository: RaidRepository,
    pub zone_repository: ZoneRepository,
}

impl Repositories {
    pub fn new(app_config: AppConfig) -> anyhow::Result<Self> {

        let manager = DuckdbConnectionManager::file(app_config.database_name)?;
        
        let pool = r2d2::Pool::builder()
            .build(manager)?;

        let config_repository = ConfigRepository::new(pool.clone());
        let file_system = FileSystem::new();
        let migration_runner = MigrationRunner::new(pool.clone(), config_repository, file_system);
        
        let confrontation_repository = ConfrontationRepository::new(pool.clone());
        let player_repository = PlayerRepository::new(pool.clone());
        let player_stats_repository = PlayerStatsRepository::new(pool.clone());
        let raid_repository = RaidRepository::new(pool.clone());
        let npc_repository = NpcRepository::new(pool.clone());
        let hp_log_repository = HpLogRepository::new(pool.clone());
        let hp_session_repository = HpSessionRepository::new(pool.clone());
        let zone_repository = ZoneRepository::new(pool.clone());

        migration_runner.run(&app_config.version)?;

        Ok(Self {
            confrontation_repository,
            hp_log_repository,
            hp_session_repository,
            npc_repository,
            player_repository,
            player_stats_repository,
            raid_repository,
            zone_repository
        })
    }
}