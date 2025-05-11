use std::sync::Arc;

use chrono::Utc;
use tauri::{AppHandle, State, command};

use crate::{
    error::AppError, models::LoadResult, services::AppReadyState, utils::get_rust_version,
};

#[command]
pub async fn load(
    app_ready_state: State<'_, Arc<AppReadyState>>,
    app_handle: AppHandle,
) -> Result<LoadResult, AppError> {
    app_ready_state.mark_ready();

    let version = app_handle.package_info().version.to_string();

    let rust_version = get_rust_version()?;

    let result = LoadResult {
        app_name: "Rust Playground".into(),
        rust_version,
        github_link: env!("CARGO_PKG_REPOSITORY").to_string(),
        loaded_on: Utc::now(),
        version,
    };

    Ok(result)
}
