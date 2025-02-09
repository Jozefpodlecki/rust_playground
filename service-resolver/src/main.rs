use std::sync::Arc;

use database_migration::DefaultDatabaseMigraiton;
use duckdb::{Connection, DuckdbConnectionManager};
use r2d2::ManageConnection;
use service_registry::ServiceRegistry;
use task_repository::{DefaultTaskRepository, TaskRepository};


mod service_registry;
mod task_repository;
mod database_migration;

fn main() {
    let mut registry = ServiceRegistry::new();

    let connection_manager = DuckdbConnectionManager::file("db.db3").unwrap();
    let pool = r2d2::Pool::new(connection_manager).unwrap();

    let shared_repository: Arc<dyn TaskRepository> = Arc::new(DefaultTaskRepository::new(pool.clone()));
    registry.register(Arc::new(DefaultDatabaseMigraiton::new(pool.clone())));
    // registry.register(shared_repository);

    let task_repository = registry.get_required::<DefaultTaskRepository>();
}
